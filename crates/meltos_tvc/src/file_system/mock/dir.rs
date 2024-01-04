use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::io::ErrorKind;

use crate::file_system::{Stat, StatType};
use crate::file_system::mock::{entry_name, MockEntry, MockEntryMut, parent_path};
use crate::file_system::mock::file::MockFile;
use crate::time::since_epoch_secs;

#[derive(Eq, PartialEq, Clone)]
pub struct MockDir {
    create_time: u64,
    update_time: u64,
    pub(crate) entries: HashMap<String, MockEntry>,
}


impl MockDir {
    #[inline(always)]
    pub fn new() -> Self {
        let create_time = since_epoch_secs();
        Self {
            create_time,
            update_time: create_time,
            entries: HashMap::default(),
        }
    }


    pub fn try_read(&mut self, path: &str) -> std::io::Result<MockEntryMut> {
        self
            .read(path)
            .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, format!("not found path={path}")))
    }


    pub fn read(&mut self, path: &str) -> Option<MockEntryMut> {
        if path.is_empty() || path == "." || path == "./" {
            Some(MockEntryMut::Dir(self))
        } else {
            Some(self.read_recursive(&path.split('/').collect::<Vec<&str>>())?)
        }
    }

    #[inline(always)]
    pub fn stat(&self) -> Stat {
        Stat {
            create_time: self.create_time,
            update_time: self.update_time,
            ty: StatType::Dir,
            size: self.entries.len() as u64,
        }
    }


    pub fn lookup_parent_dir(&mut self, path: &str) -> Option<&mut MockDir> {
        let parent_dir = parent_path(path)?;
        if self.exists(&parent_dir) {
            Some(self.read(&parent_dir).unwrap().dir().unwrap())
        } else {
            None
        }
    }


    pub fn create_dir(&mut self, path: &str) -> &mut MockDir {
        let mut ps: VecDeque<&str> = path.split("/").collect();
        self.update_time_recursive(&path);
        let mut dir = self;
        while let Some(name) = ps.pop_front() {
            if dir.entries.contains_key(name) {
                dir = dir
                    .read(name)
                    .and_then(|entry| entry.dir().ok())
                    .unwrap();
            } else {
                dir = dir._create_dir(name);
            }
        }

        dir
    }


    pub fn write_file(&mut self, path: impl Into<String>, buf: &[u8]) {
        let path: String = path.into();
        let mut ps: Vec<&str> = path.split('/').collect();
        if let Some(parent) = self.lookup_parent_dir(&path) {
            let name = ps.pop().unwrap();
            parent._write_file(name.to_string(), buf.to_vec());
        } else if let Some(parent) = parent_path(&path){
            let parent = self.create_dir(&parent);
            parent._write_file(entry_name(&path), buf.to_vec());
        }else{
            self._write_file(entry_name(&path), buf.to_vec());
        }
        self.update_time_recursive(&path);
    }


    #[inline(always)]
    fn update_time_recursive(&mut self, path: &str) {
        let ps: Vec<&str> = path.split('/').collect();
        let update_time = since_epoch_secs();
        self._update_time_recursive(update_time, &ps);
    }


    #[inline(always)]
    fn _update_time_recursive(&mut self, update_time: u64, path: &[&str]) {
         println!("_update_time_recursive {path:?}");
        self.update_time = update_time;
        if path.is_empty() {
            return;
        }
        let next_name = path[0];

        let Some(entry) = self.read(next_name) else {
            println!("RETURN");
            return;
        };
        match entry {
            MockEntryMut::File(file) => {
                println!("DDDD");
                file.update_time = update_time;
            }
            MockEntryMut::Dir(dir) => {
                dir._update_time_recursive(update_time, &path[1..]);
            }
        }
    }


    #[inline(always)]
    fn _write_file(&mut self, entry_name: String, buf: Vec<u8>) {
        if let Some(file) = self
            .entries
            .get_mut(&entry_name)
            .and_then(|entry| entry.file_mut().ok())
        {
            file.buf = buf;
        } else {
            self.entries.insert(entry_name, MockEntry::File(MockFile::new(buf)));
        }
    }


    pub fn all_files(&self, parent_path: Option<String>) -> Vec<String> {
        let mut files = Vec::new();
        for (name, entry) in self.entries.iter() {
            if let Ok(dir) = entry.dir_ref() {
                let new_parent = parent_path
                    .as_ref()
                    .map(|p| format!("{p}/{name}"))
                    .unwrap_or(name.to_string());

                files.extend(dir.all_files(Some(new_parent)));
            } else {
                let file_path = parent_path
                    .as_ref()
                    .map(|p| format!("{p}/{name}"))
                    .unwrap_or(name.to_string());
                files.push(file_path);
            }
        }

        files
    }


    fn _create_dir(&mut self, path: &str) -> &mut MockDir {
        self.entries.insert(path.to_string(), MockEntry::Dir(MockDir::new()));
        self.entries.get_mut(path).unwrap().dir_mut().unwrap()
    }


    #[inline(always)]
    pub fn exists(&mut self, path: &str) -> bool {
        self.read(path).is_some()
    }

    fn read_recursive(&mut self, path: &[&str]) -> Option<MockEntryMut> {
        let name = path[0];
        let entry = self.read_entry(name)?;
        if path.len() == 1 || entry.stat().is_file() {
            return Some(entry);
        }

        let dir = entry.dir().unwrap();
        dir.read_recursive(&path[1..])
    }


    fn read_entry(&mut self, name: &str) -> Option<MockEntryMut> {
        if name == "./" || name == "." {
            return Some(MockEntryMut::Dir(self));
        }

        println!("entry name = {name}\n {self:?}");

        let entry = self.entries.get(name)?.clone();
         println!("dadadad entry name = {name}");
        if entry.stat().is_file() {
            Some(MockEntryMut::File(self.entries.get_mut(name)?.file_mut().unwrap()))
        } else {
            Some(MockEntryMut::Dir(self.entries.get_mut(name)?.dir_mut().unwrap()))
        }
    }
}


impl Debug for MockDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for key in self.entries.keys() {
            f.write_fmt(format_args!("{key}\n"))?;
        }
        Ok(())
    }
}


impl Default for MockDir {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
