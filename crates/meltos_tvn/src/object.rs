use std::io;
use std::io::ErrorKind;

use serde::{Deserialize, Serialize};

use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;
use meltos_util::macros::{Deref, Display};

use crate::io::{FilePath, OpenIo, TvnIo};

#[derive(Debug, Clone)]
pub struct ObjectIo<Open, Io>(TvnIo<Open, Io>)
    where
        Open: OpenIo<Io>,
        Io: io::Read + io::Write;

impl<Open, Io> Default for ObjectIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvnIo::default())
    }
}


impl<Open, Io> ObjectIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(open: Open) -> ObjectIo<Open, Io> {
        Self(TvnIo::new(open))
    }


    pub fn try_read(&self, object_hash: &ObjectHash) -> io::Result<Object> {
        self.read(object_hash).and_then(|obj| match obj {
            Some(obj) => Ok(obj),
            None => Err(std::io::Error::new(ErrorKind::NotFound, format!("not found object: hash={object_hash}")))
        })
    }


    pub fn read(&self, object_hash: &ObjectHash) -> io::Result<Option<Object>> {
        let Some(buf) = self.0.read_to_end(&format!("./.meltos/objects/{}", object_hash))? else {
            return Ok(None);
        };

        Ok(Some(Object::expand(CompressedBuf(buf))?))
    }


    pub fn write(&self, obj: &Object) -> io::Result<()> {
        self.0
            .create(&format!("./.meltos/objects/{}", &obj.hash))?
            .write_all(&obj.compressed_buf)?;
        Ok(())
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ObjectMeta {
    pub file_path: FilePath,
    pub obj: Object,
}


impl From<(FilePath, Object)> for ObjectMeta {
    #[inline(always)]
    fn from(value: (FilePath, Object)) -> Self {
        Self{
            file_path: value.0,
            obj: value.1
        }
    }
}


impl ObjectMeta {
    pub fn new(file_path: FilePath, buf: Vec<u8>) -> std::io::Result<Self> {
        Ok(Self {
            file_path,
            obj: Object::compress(buf)?,
        })
    }


    #[inline]
    pub const fn hash(&self) -> &ObjectHash {
        &self.obj.hash
    }


    #[inline]
    pub const fn compressed_buf(&self) -> &CompressedBuf {
        &self.obj.compressed_buf
    }


    #[inline]
    pub fn buf(&self) -> &[u8] {
        &self.obj.buf
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Object {
    pub hash: ObjectHash,
    pub compressed_buf: CompressedBuf,
    pub buf: Vec<u8>,
}


impl Object {
    pub fn compress(buf: Vec<u8>) -> std::io::Result<Self> {
        Ok(Self {
            hash: ObjectHash::new(&buf),
            compressed_buf: CompressedBuf(Gz.encode(&buf)?),
            buf,
        })
    }

    pub fn expand(compressed_buf: CompressedBuf) -> std::io::Result<Self> {
        let buf = Gz.decode(&compressed_buf)?;
        Ok(Self {
            hash: ObjectHash::new(&buf),
            buf,
            compressed_buf,
        })
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Display)]
pub struct ObjectHash(pub String);


impl ObjectHash {
    #[inline]
    pub fn new(buf: &[u8]) -> Self {
        Self(meltos_util::hash::hash(buf))
    }
}


#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Deref)]
pub struct CompressedBuf(pub Vec<u8>);


#[cfg(test)]
mod tests {
    use std::io::Write;

    use meltos_util::compression::CompressionBuf;
    use meltos_util::compression::gz::Gz;

    use crate::io::{OpenIo, TvnIo};
    use crate::io::mock::MockOpenIo;
    use crate::object::{Object, ObjectIo};
    use crate::workspace::WorkspaceIo;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let mock = MockOpenIo::default();
        mock.create("test/hello.txt")
            .unwrap()
            .write_all(buf)
            .unwrap();

        let io = ObjectIo::new(mock.clone());
        let workspace = WorkspaceIo(TvnIo::new(mock.clone()));
        let mut objs = workspace.convert_to_objs("test/hello.txt").unwrap();
        let meta = objs.next().unwrap().unwrap();
        io.write(&meta.obj).unwrap();

        let hello_buf = mock
            .try_read_to_end(&format!(
                "./.meltos/objects/{}",
                meltos_util::hash::hash(buf)
            ))
            .unwrap();
        assert_eq!(hello_buf, Gz.encode(buf).unwrap());
    }


    #[test]
    fn read_obj() {
        let mock = MockOpenIo::default();
        let io = ObjectIo::new(mock.clone());
        let obj = Object::compress(b"hello world!".to_vec()).unwrap();
        io.write(&obj).unwrap();
        assert_eq!(io.read(&obj.hash).unwrap(), Some(obj));
    }
}
