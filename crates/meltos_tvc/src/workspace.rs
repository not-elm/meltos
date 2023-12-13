use std::io;
use std::path::Path;

use crate::io::{OpenIo, TvcIo};
use crate::object::Object;

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Open, Io>(pub TvcIo<Open, Io>)
    where
        Open: OpenIo<Io>,
        Io: io::Read + io::Write;


impl<Open, Io> WorkspaceIo<Open, Io>
    where
        Open: OpenIo<Io> ,
        Io: io::Read + io::Write,
{
    pub fn read_to_object(&self, path: impl AsRef<Path>) -> std::io::Result<Object> {
        let buf = self.0.try_read_to_end(path)?;
        Ok(Object::new(buf))
    }
}


impl<Open, Io> Default for WorkspaceIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvcIo::default())
    }
}