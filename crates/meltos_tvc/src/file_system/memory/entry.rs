use crate::file_system::memory::entry::dir::MemoryDir;
use crate::file_system::memory::entry::file::MemoryFile;
use crate::file_system::Stat;
use std::fmt::{Debug, Formatter};
pub mod dir;
pub mod file;

#[derive(Clone)]
pub enum MemoryEntry {
    File(MemoryFile),
    Dir(MemoryDir),
}

impl Debug for MemoryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryEntry::Dir(dir) => f.write_fmt(format_args!("{dir:#?}")),
            MemoryEntry::File(file) => f.write_fmt(format_args!("{file:#?}")),
        }
    }
}

impl MemoryEntry {
    #[inline]
    pub fn stat(&self) -> Stat {
        match self {
            MemoryEntry::File(file) => file.stat(),
            MemoryEntry::Dir(dir) => dir.stat(),
        }
    }

    pub fn file(self) -> std::io::Result<MemoryFile> {
        match self {
            MemoryEntry::File(file) => Ok(file),
            MemoryEntry::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn file_mut(&mut self) -> std::io::Result<&mut MemoryFile> {
        match self {
            MemoryEntry::File(file) => Ok(file),
            MemoryEntry::Dir(_) => Err(std::io::Error::other("expect file mut bad was dir")),
        }
    }

    pub fn dir_mut(&mut self) -> std::io::Result<&mut MemoryDir> {
        match self {
            MemoryEntry::Dir(dir) => Ok(dir),
            MemoryEntry::File(_) => Err(std::io::Error::other("expect dir mut bad was file.")),
        }
    }

    pub fn dir_ref(&self) -> std::io::Result<&MemoryDir> {
        match self {
            MemoryEntry::Dir(dir) => Ok(dir),
            MemoryEntry::File(_) => Err(std::io::Error::other("expect dir ref bad was file.")),
        }
    }

    pub fn dir(self) -> std::io::Result<MemoryDir> {
        match self {
            MemoryEntry::Dir(dir) => Ok(dir),
            MemoryEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }
}
