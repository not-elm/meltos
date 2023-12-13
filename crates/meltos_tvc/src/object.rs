use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

use crate::io::{OpenIo, TvcIo};

#[derive(Debug, Clone)]
pub struct ObjectIo<Open, Io>(TvcIo<Open, Io>)
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write;

impl<Open, Io> Default for ObjectIo<Open, Io>
where
    Open: OpenIo<Io> + Default,
    Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvcIo::default())
    }
}


impl<Open, Io> ObjectIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(open: Open) -> ObjectIo<Open, Io> {
        Self(TvcIo::new(open))
    }


    pub fn write(&self, object: &Object) -> io::Result<()> {
        const DIR_PATH: &str = "./.meltos/objects/";
        let path: &Path = DIR_PATH.as_ref();
        let path = path.join(&object.hash.0);
        self.0.create(path)?.write_all(&object.buf)?;
        Ok(())
    }
}



#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Object {
    pub hash: ObjectHash,
    pub buf: Vec<u8>,
}


impl Object {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            hash: ObjectHash(meltos_util::hash::hash(&buf)),
            buf,
        }
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ObjectHash(pub String);


#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::io::mock::MockOpenIo;
    use crate::io::{OpenIo, TvcIo};
    use crate::object::ObjectIo;
    use crate::workspace::WorkspaceIo;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let open = MockOpenIo::default();
        open.create("test/hello.txt").unwrap().write_all(buf).unwrap();

        let io = ObjectIo::new(open.clone());
        let hello_obj = WorkspaceIo(TvcIo::new(open.clone())).read_to_object("test/hello.txt").unwrap();
        io.write(&hello_obj).unwrap();

        let hello_buf = open
            .try_read_to_end(format!(
                "./.meltos/objects/{}",
                meltos_util::hash::hash(buf)
            ))
            .unwrap();
        assert_eq!(hello_buf, buf);
    }
}
