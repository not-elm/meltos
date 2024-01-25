use crate::file_system::mock::entry::dir::MockDir;
use crate::file_system::mock::entry::file::MockFile;
use crate::file_system::Stat;
pub mod dir;
pub mod file;

#[derive(Debug, Clone)]
pub enum MockEntry {
    File(MockFile),
    Dir(MockDir),
}

impl MockEntry {
    #[inline]
    pub fn stat(&self) -> Stat {
        match self {
            MockEntry::File(file) => file.stat(),
            MockEntry::Dir(dir) => dir.stat(),
        }
    }

    pub fn file(self) -> std::io::Result<MockFile> {
        match self {
            MockEntry::File(file) => Ok(file),
            MockEntry::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn file_mut(&mut self) -> std::io::Result<&mut MockFile> {
        match self {
            MockEntry::File(file) => Ok(file),
            MockEntry::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn dir_mut(&mut self) -> std::io::Result<&mut MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }

    pub fn dir_ref(&self) -> std::io::Result<&MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }

    pub fn dir(self) -> std::io::Result<MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }
}
