use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

pub mod file;
pub(crate) mod mock;
pub mod compression;


pub trait OpenIo<Io: std::io::Read + std::io::Write> {
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Option<Io>>;


    fn create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Io>;


    fn read_to_end<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Option<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.open(path)? {
            Some(mut io) => {
                io.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
            None => Ok(None)
        }
    }

    fn try_read_to_end(&self, path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
        self.read_to_end(path)
            .and_then(|buf| {
                match buf {
                    Some(buf) => Ok(buf),
                    None => Err(std::io::Error::new(ErrorKind::NotFound, "file not found"))
                }
            })
    }


    fn write<P: AsRef<Path>>(&self, path: P, buf: &[u8]) -> std::io::Result<()> {
        self.create(path)?.write_all(buf)
    }
}


#[derive(Debug)]
pub struct TvcIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Read + std::io::Write,
{
    open: Open,
    _io: PhantomData<Io>,
}

impl<Open, Io> TvcIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Read + std::io::Write,
{
    #[inline]
    pub const fn new(open: Open) -> TvcIo<Open, Io> {
        Self {
            open,
            _io: PhantomData,
        }
    }
}


impl<Open, Io> Default for TvcIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn default() -> TvcIo<Open, Io> {
        Self {
            open: Open::default(),
            _io: PhantomData,
        }
    }
}


impl<Open, Io> Clone for TvcIo<Open, Io>
    where
        Open: OpenIo<Io> + Clone,
        Io: std::io::Read + std::io::Write,
{
    fn clone(&self) -> Self {
        Self::new(self.open.clone())
    }
}


impl<Open, Io> OpenIo<Io> for TvcIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Option<Io>> {
        self.open.open(path)
    }

    #[inline]
    fn create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Io> {
        self.open.create(path)
    }
}


#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct FilePath(pub String);
impl_string_new_type!(FilePath);


impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}



