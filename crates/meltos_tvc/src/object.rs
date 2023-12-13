use serde::{Deserialize, Serialize};
use std::io;
use std::marker::PhantomData;
use std::path::Path;

use crate::io::OpenIo;

pub struct ObjectIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    open: Open,
    _io: PhantomData<Io>,
}


impl<Open, Io> ObjectIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(open: Open) -> ObjectIo<Open, Io> {
        Self {
            open,
            _io: PhantomData,
        }
    }


    pub fn stage<P: AsRef<Path>>(&self, from: P) -> std::io::Result<()> {
        let file_buf = self.open.read_to_end(from)?;
        self.write(file_buf)
    }

    pub fn write(&self, buf: Vec<u8>) -> io::Result<()> {
        let object = Object::new(buf)?;
        const DIR_PATH: &str = "./.meltos/objects/";

        let path: &Path = DIR_PATH.as_ref();
        let path = path.join(&object.hash.0);
        self.open.open(path)?.write_all(&object.buf)?;

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Object {
    pub hash: ObjectHash,
    pub buf: Vec<u8>,
}


impl Object {
    pub fn new(buf: Vec<u8>) -> std::io::Result<Object> {
        Ok(Self {
            hash: ObjectHash(meltos_util::hash::hash(&buf)),
            buf,
        })
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ObjectHash(pub String);


#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::io::mock::MockOpenIo;
    use crate::io::OpenIo;
    use crate::object::ObjectIo;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let open = MockOpenIo::default();
        open.open("test/hello.txt").unwrap().write_all(buf).unwrap();

        let io = ObjectIo::new(open.clone());
        io.stage("test/hello.txt").unwrap();

        let hello_buf = open
            .read_to_end(format!(
                "./.meltos/objects/{}",
                meltos_util::hash::hash(buf)
            ))
            .unwrap();
        assert_eq!(hello_buf, buf);
    }
}
