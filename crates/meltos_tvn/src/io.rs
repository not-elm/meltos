use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

pub mod file;
pub(crate) mod mock;


pub trait OpenIo<Io: std::io::Read + std::io::Write> {
    fn open_file(&self, path: &str) -> std::io::Result<Option<Io>>;


    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>>;


    fn create(&self, path: &str) -> std::io::Result<Io>;


    fn delete(&self, path: &str) -> std::io::Result<()>;


    fn read_to_end(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.open_file(path)? {
            Some(mut io) => {
                io.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
            None => Ok(None),
        }
    }

    fn try_read_to_end(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.read_to_end(path).and_then(|buf| {
            match buf {
                Some(buf) => Ok(buf),
                None => Err(std::io::Error::new(ErrorKind::NotFound, "file not found")),
            }
        })
    }


    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.create(path)?.write_all(buf)
    }
}


#[derive(Debug)]
pub struct TvnIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Read + std::io::Write,
{
    open: Open,
    _io: PhantomData<Io>,
}

impl<Open, Io> TvnIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    pub const fn new(open: Open) -> TvnIo<Open, Io> {
        Self {
            open,
            _io: PhantomData,
        }
    }
}


impl<Open, Io> Default for TvnIo<Open, Io>
where
    Open: OpenIo<Io> + Default,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn default() -> TvnIo<Open, Io> {
        Self {
            open: Open::default(),
            _io: PhantomData,
        }
    }
}


impl<Open, Io> Clone for TvnIo<Open, Io>
where
    Open: OpenIo<Io> + Clone,
    Io: std::io::Read + std::io::Write,
{
    fn clone(&self) -> Self {
        Self::new(self.open.clone())
    }
}


impl<Open, Io> OpenIo<Io> for TvnIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn open_file(&self, path: &str) -> std::io::Result<Option<Io>> {
        self.open.open_file(path)
    }

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        self.open.all_file_path(path)
    }

    #[inline]
    fn create(&self, path: &str) -> std::io::Result<Io> {
        self.open.create(path)
    }

    #[inline]
    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.open.delete(path)
    }
}


#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
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
