use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

pub mod file;
pub(crate) mod mock;

pub trait FileSystem<Io: std::io::Read + std::io::Write> {
    fn open_file(&self, path: &str) -> std::io::Result<Option<Io>>;

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>>;

    fn create(&self, path: &str) -> std::io::Result<Io>;

    fn delete(&self, path: &str) -> std::io::Result<()>;


    fn try_read(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.read(path).and_then(|buf| {
            match buf {
                Some(buf) => Ok(buf),
                None => Err(std::io::Error::new(ErrorKind::NotFound, "file not found")),
            }
        })
    }

    fn read(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.open_file(path)? {
            Some(mut io) => {
                io.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
            None => Ok(None),
        }
    }

    fn project_already_initialized(&self) -> std::io::Result<bool> {
        let files = self.all_file_path("./.meltos")?;
        Ok(!files.is_empty())
    }


    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.create(path)?.write_all(buf)
    }
}

#[derive(Debug)]
pub struct FsIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Read + std::io::Write,
{
    fs: Fs,
    _io: PhantomData<Io>,
}

impl<Fs, Io> FsIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    pub const fn new(fs: Fs) -> FsIo<Fs, Io> {
        Self {
            fs,
            _io: PhantomData,
        }
    }
}

impl<Fs, Io> Default for FsIo<Fs, Io>
where
    Fs: FileSystem<Io> + Default,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn default() -> FsIo<Fs, Io> {
        Self {
            fs: Fs::default(),
            _io: PhantomData,
        }
    }
}

impl<Fs, Io> Clone for FsIo<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Read + std::io::Write,
{
    fn clone(&self) -> Self {
        Self::new(self.fs.clone())
    }
}

impl<Fs, Io> FileSystem<Io> for FsIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn open_file(&self, path: &str) -> std::io::Result<Option<Io>> {
        self.fs.open_file(path)
    }

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        self.fs.all_file_path(path)
    }

    #[inline]
    fn create(&self, path: &str) -> std::io::Result<Io> {
        self.fs.create(path)
    }

    #[inline]
    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(path)
    }
}


#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct FilePath(pub String);
impl_string_new_type!(FilePath);

impl FilePath {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().to_str().unwrap().to_string())
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<String> for FilePath {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
