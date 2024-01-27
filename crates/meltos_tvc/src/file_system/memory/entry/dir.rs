use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Pointer, Write};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use meltos_util::path::AsUri;

use crate::file_system::{Stat, StatType};
use crate::file_system::memory::{as_schemes, entry_name, parent_path};
use crate::file_system::memory::entry::file::MemoryFile;
use crate::file_system::memory::entry::MemoryEntry;
use crate::time::since_epoch_secs;

#[repr(transparent)]
#[derive(Clone)]
pub struct MemoryDir(Arc<RwLock<MemoryDirInner>>);

impl Debug for MemoryDir{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let entry = self.0.read().unwrap();
        f.write_fmt(format_args!("{entry:#?}"))
    }
}


impl MemoryDir {
    #[inline(always)]
    pub fn root() -> Self {
        Self::new("", None)
    }

    #[inline(always)]
    pub fn name(&self) -> String {
        self.0.read().unwrap().name.clone()
    }

    #[inline(always)]
    pub fn new(name: impl Into<String>, parent: Option<MemoryDir>) -> Self {
        let dir = Self(Arc::new(RwLock::new(MemoryDirInner::new(name.into()))));
        dir.set_dir(".", dir.clone());
        if let Some(parent) = parent {
            dir.set_dir("..", parent);
        }
        dir
    }

    #[inline(always)]
    pub fn stat(&self) -> Stat {
        let inner = self.0.read().unwrap();
        inner.stat()
    }

    #[inline(always)]
    pub fn set_dir(&self, entry_name: impl Into<String>, dir: MemoryDir) {
        self.0
            .write()
            .unwrap()
            .entries
            .insert(entry_name.into(), MemoryEntry::Dir(dir));
    }

    #[inline(always)]
    pub fn entry_names(&self) -> Vec<String> {
        self.0
            .read()
            .unwrap()
            .entries
            .keys()
            .filter(|name| ignore_name(name))
            .map(|name| self.as_absolute_uri(name))
            .collect()
    }

    #[inline(always)]
    pub fn delete(&self, entry_name: &str) {
        self.0.write().unwrap().entries.remove(entry_name);
    }

    pub fn try_read(&self, path: &str) -> std::io::Result<MemoryEntry> {
        self.read(path).ok_or_else(|| {
            std::io::Error::new(ErrorKind::NotFound, format!("not found path={path}"))
        })
    }

    pub fn read(&self, path: &str) -> Option<MemoryEntry> {
        self.read_recursive(&as_schemes(path))
    }

    pub fn write_file(&self, path: impl Into<String>, buf: &[u8]) {
        let path: String = path.into();
        if let Some(parent) = self.lookup_parent_dir(&path) {
            parent._write_file(entry_name(&path), buf.to_vec());
        } else if let Some(parent) = parent_path(&path) {
            let parent = self.create_dir(&parent);
            parent._write_file(entry_name(&path), buf.to_vec());
        } else {
            self._write_file(entry_name(&path), buf.to_vec());
        }
        self.update_time_recursive(&path);
    }

    pub fn create_dir(&self, path: &str) -> MemoryDir {
        let mut dir = self.clone();
        dir.update_time_recursive(path);
        for name in as_schemes(path) {
            if dir.0.read().unwrap().entries.contains_key(&name) {
                dir = dir.read(&name).and_then(|entry| entry.dir().ok()).unwrap();
            } else {
                let mut inner = dir.0.write().unwrap();
                let child = MemoryDir::new(name.to_string(), Some(dir.clone()));
                inner
                    .entries
                    .insert(name.to_string(), MemoryEntry::Dir(child.clone()));
                let child = inner
                    .entries
                    .get_mut(&name)
                    .unwrap()
                    .dir_mut()
                    .unwrap()
                    .clone();
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

    pub fn lookup_parent_dir(&self, path: &str) -> Option<MemoryDir> {
        let parent_dir = parent_path(path)?;
        if self.exists(&parent_dir) {
            Some(self.read(&parent_dir).unwrap().dir().unwrap())
        } else {
            None
        }
    }

    pub fn all_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        let lock = self.0.read().unwrap();
        let entries: Vec<(String, MemoryEntry)> = lock
            .entries
            .iter()
            .filter(|(name, _)| ignore_name(name))
            .map(|(name, entry)| (name.to_string(), entry.clone()))
            .collect();
        drop(lock);
        for (name, entry) in entries {
            if let Ok(dir) = entry.dir_ref() {
                files.extend(dir.all_files());
            } else {
                files.push(self.as_absolute_uri(&name));
            }
        }

        files
    }

    fn as_absolute_uri(&self, entry_name: &str) -> String {
        self.parent_path_buf().join(entry_name).as_uri()
    }

    fn parent_path_buf(&self) -> PathBuf {
        if let Some(parent) = self
            .read_entry("..")
            .and_then(|p| p.dir().ok())
            .map(|dir| dir.parent_path_buf())
        {
            parent.join(self.name())
        } else {
            Path::new(&self.name()).to_path_buf()
        }
    }

    fn read_recursive(&self, path: &[String]) -> Option<MemoryEntry> {
        if path.is_empty() {
            return None;
        }

        let name = &path[0];
        let entry = self.read_entry(name)?;
        if path.len() == 1 || entry.stat().is_file() {
            return Some(entry);
        }

        let dir = entry.dir().unwrap();
        dir.read_recursive(&path[1..])
    }

    fn read_entry(&self, name: &str) -> Option<MemoryEntry> {
        let dir = self.0.read().unwrap();
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
        let mut inner = self.0.write().unwrap();
        if let Some(file) = inner
            .entries
            .get_mut(&entry_name)
            .and_then(|entry| entry.file_mut().ok())
        {
            file.write(buf);
        } else {
            inner
                .entries
                .insert(entry_name, MemoryEntry::File(MemoryFile::new(buf)));
        }
    }

    #[inline(always)]
    fn _update_time_recursive(&self, update_time: u64, path: &[&str]) {
        self.0.write().unwrap().update_time = update_time;
        if path.is_empty() {
            return;
        }
        let next_name = path[0];

        let Some(entry) = self.read(next_name) else {
            return;
        };
        match entry {
            MemoryEntry::File(file) => {
                file.set_update_time(update_time);
            }
            MemoryEntry::Dir(dir) => {
                dir._update_time_recursive(update_time, &path[1..]);
            }
        }
    }
}

#[derive(Clone)]
struct MemoryDirInner {
    name: String,
    create_time: u64,
    update_time: u64,
    entries: HashMap<String, MemoryEntry>,
}

impl MemoryDirInner {
    pub fn new(name: String) -> Self {
        let create_time = since_epoch_secs();
        Self {
            name,
            create_time,
            update_time: create_time,
            entries: HashMap::default(),
        }
    }

    pub fn stat(&self) -> Stat {
        Stat {
            create_time: self.create_time,
            update_time: self.update_time,
            ty: StatType::Dir,
            size: self.entries.len() as u64 - 2, // `.`と`..`を除く
        }
    }
}

impl Debug for MemoryDirInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("MemoryDir");
        for (key, entry) in self.entries.iter() {
            if key == "." || key == ".." {
                continue;
            }
            if let MemoryEntry::Dir(dir) = entry {
                fmt.field(key, &entry);
            } else {
                fmt.field(key, &entry.stat());
            }
        }
        fmt.finish()
    }
}

#[inline(always)]
fn ignore_name(name: impl AsRef<str>) -> bool {
    let name: &str = name.as_ref();
    !(name == "." || name == "..")
}

#[cfg(test)]
mod tests {
    use crate::file_system::memory::entry::dir::MemoryDir;

    #[test]
    fn it_ignored_mine() {
        let root = MemoryDir::root();
        let entries = root
            .0
            .read()
            .unwrap()
            .entries
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        // 実際には自身(root)を表すエントリーが存在する
        assert_eq!(entries, vec!["."]);

        // entry_namesから返される一覧からは除外される
        let names = root.entry_names();
        assert_eq!(names.len(), 0);
    }

    #[test]
    fn without_slash() {
        let root = MemoryDir::root();
        root.write_file("hello.txt", b"hello");

        assert_eq!(
            root.read(".").unwrap().dir_ref().unwrap().all_files(),
            vec!["hello.txt"]
        );
    }

    #[test]
    fn without_slash_with_in_dir() {
        let root = MemoryDir::root();
        root.write_file("a.txt", b"hello");
        root.write_file("dir/b.txt", b"hello");
        let mut files = root.read(".").unwrap().dir_ref().unwrap().all_files();
        files.sort();
        assert_eq!(
            files,
            vec!["a.txt", "dir/b.txt"]
        );
    }
}
