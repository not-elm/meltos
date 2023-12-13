use std::marker::PhantomData;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

mod file;
pub(crate) mod mock;


pub trait OpenIo<Io: std::io::Read + std::io::Write> {
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Io>;


    fn read_to_end<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.open(path)?.read_to_end(&mut buf)?;
        Ok(buf)
    }


    fn write<P: AsRef<Path>>(&self, path: P, buf: &[u8]) -> std::io::Result<()> {
        self.open(path)?.write_all(buf)
    }
}


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


impl<Open, Io> OpenIo<Io> for TvcIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Read + std::io::Write,
{
    #[inline]
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Io> {
        self.open.open(path)
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
