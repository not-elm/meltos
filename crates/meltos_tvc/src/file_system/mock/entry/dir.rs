use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

use crate::file_system::{Stat, StatType};
use crate::file_system::mock::{as_schemes, entry_name, parent_path};
use crate::file_system::mock::entry::file::MockFile;
use crate::file_system::mock::entry::MockEntry;
use crate::time::since_epoch_secs;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct MockDir(Arc<Mutex<MockDirInner>>);

impl MockDir {
    #[inline(always)]
    pub fn new(name: impl Into<String>, parent: Option<MockDir>) -> Self {
        let dir = Self(Arc::new(Mutex::new(MockDirInner::new(name.into()))));
        dir.0.lock().unwrap().entries.insert(".".to_string(), MockEntry::Dir(dir.clone()));
        if let Some(parent) = parent {
            dir.0.lock().unwrap().entries.insert("..".to_string(), MockEntry::Dir(parent));
        }
        dir
    }

    #[inline(always)]
    pub fn stat(&self) -> Stat {
        let inner = self.0.lock().unwrap();
        Stat {
            create_time: inner.create_time,
            update_time: inner.update_time,
            ty: StatType::Dir,
            size: inner.entries.len() as u64 - 2, // `.`と`..`を除く
        }
    }


    #[inline(always)]
    pub fn set_dir(&self, entry_name: impl Into<String>, dir: MockDir) {
        self.0.lock().unwrap().entries.insert(entry_name.into(), MockEntry::Dir(dir));
    }


    #[inline(always)]
    pub fn entry_names(&self) -> Vec<String> {
        self
            .0
            .lock()
            .unwrap()
            .entries
            .keys()
            .filter(|name| ignore_name(name))
            .cloned()
            .collect()
    }


    #[inline(always)]
    pub fn delete(&self, entry_name: &str) {
        self.0.lock().unwrap().entries.remove(entry_name);
    }


    pub fn try_read(&self, path: &str) -> std::io::Result<MockEntry> {
        self.read(path).ok_or_else(|| {
            std::io::Error::new(ErrorKind::NotFound, format!("not found path={path}"))
        })
    }

    pub fn read(&self, path: &str) -> Option<MockEntry> {
        self.read_recursive(&as_schemes(path))
    }

    pub fn write_file(&self, path: impl Into<String>, buf: &[u8]) {
        let path: String = path.into();
        let mut schemes = as_schemes(&path);
        if let Some(parent) = self.lookup_parent_dir(&path) {
            let name = schemes.pop().unwrap();
            parent._write_file(name.to_string(), buf.to_vec());
        } else if let Some(parent) = parent_path(&path) {
            let parent = self.create_dir(&parent);
            parent._write_file(entry_name(&path), buf.to_vec());
        } else {
            self._write_file(entry_name(&path), buf.to_vec());
        }
        self.update_time_recursive(&path);
    }


    pub fn create_dir(&self, path: &str) -> MockDir {
        let mut schemes = as_schemes(path).into_iter().collect::<VecDeque<&str>>();
        let mut dir = self.clone();
        dir.update_time_recursive(path);
        while let Some(name) = schemes.pop_front() {
            if dir.0.lock().unwrap().entries.contains_key(name) {
                dir = dir
                    .read(name)
                    .and_then(|entry| entry.dir().ok())
                    .unwrap();
            } else {
                let mut inner = dir.0.lock().unwrap();
                let child = MockDir::new(name.to_string(), Some(dir.clone()));
                inner.entries.insert(name.to_string(), MockEntry::Dir(child.clone()));
                let child = inner.entries.get_mut(name).unwrap().dir_mut().unwrap().clone();
                drop(inner);
                dir = child;
            }
        }

        dir
    }


    #[inline(always)]
    pub fn exists(&self, path: &str) -> bool {
        self.read(path).is_some()
    }

    pub fn lookup_parent_dir(&self, path: &str) -> Option<MockDir> {
        let parent_dir = parent_path(path)?;
        if self.exists(&parent_dir) {
            Some(self.read(&parent_dir).unwrap().dir().unwrap())
        } else {
            None
        }
    }

    pub fn all_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        let lock = self
            .0
            .lock()
            .unwrap();
        let entries: Vec<(String, MockEntry)> = lock
            .entries
            .iter()
            .filter(|(name, _)| ignore_name(name))
            .map(|(name, entry)|(name.to_string(), entry.clone()))
            .collect();
        drop(lock);
        for (name, entry) in entries {
            if let Ok(dir) = entry.dir_ref() {
                files.extend(dir.all_files());
            } else {
                let file_path = self
                    .parent_path()
                    .as_ref()
                    .map(|p| format!("{p}/{name}"))
                    .unwrap_or(name.to_string());
                files.push(file_path);
            }
        }

        files
    }

    fn parent_path(&self) -> Option<String> {
        if let Some(parent) = self
            .read_entry("..")
            .and_then(|p| p.dir().ok())
            .and_then(|dir| dir.parent_path()) {

            Some(format!("{parent}/{}", self.0.lock().unwrap().name))
        } else {
            Some(self.0.lock().unwrap().name.clone())
        }
    }

    fn read_recursive(&self, path: &[&str]) -> Option<MockEntry> {
        if path.is_empty() {
            return None;
        }

        let name = path[0];
        let entry = self.read_entry(name)?;

        if path.len() == 1 || entry.stat().is_file() {
            return Some(entry);
        }

        let dir = entry.dir().unwrap();
        dir.read_recursive(&path[1..])
    }

    fn read_entry(&self, name: &str) -> Option<MockEntry> {
        let dir = self.0.lock().unwrap();
        let entry = dir.entries.get(name)?.clone();
        Some(entry)
    }

    #[inline(always)]
    fn update_time_recursive(&self, path: &str) {
        let ps: Vec<&str> = path.split('/').collect();
        let update_time = since_epoch_secs();
        self._update_time_recursive(update_time, &ps);
    }


    #[inline(always)]
    fn _write_file(&self, entry_name: String, buf: Vec<u8>) {
        let mut inner = self.0.lock().unwrap();
        if let Some(file) = inner
            .entries
            .get_mut(&entry_name)
            .and_then(|entry| entry.file_mut().ok())
        {
            file.write(buf);
        } else {
            inner
                .entries
                .insert(entry_name, MockEntry::File(MockFile::new(buf)));
        }
    }


    #[inline(always)]
    fn _update_time_recursive(&self, update_time: u64, path: &[&str]) {
        self.0.lock().unwrap().update_time = update_time;
        if path.is_empty() {
            return;
        }
        let next_name = path[0];

        let Some(entry) = self.read(next_name) else {
            return;
        };
        match entry {
            MockEntry::File(file) => {
                file.set_update_time(update_time);
            }
            MockEntry::Dir(dir) => {
                dir._update_time_recursive(update_time, &path[1..]);
            }
        }
    }
}

#[derive(Clone)]
struct MockDirInner {
    pub name: String,
    create_time: u64,
    update_time: u64,
    pub(crate) entries: HashMap<String, MockEntry>,
}


impl MockDirInner {
    pub fn new(name: String) -> Self {
        let create_time = since_epoch_secs();
        Self {
            name,
            create_time,
            update_time: create_time,
            entries: HashMap::default(),
        }
    }
}


impl Debug for MockDirInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for key in self.entries.keys() {
            f.write_fmt(format_args!("{key}\n"))?;
        }
        Ok(())
    }
}


#[inline(always)]
fn ignore_name(name: impl AsRef<str>) -> bool {
    let name: &str = name.as_ref();
    !(name == "." || name == "..")
}


