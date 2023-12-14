use std::io;

use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::{CompressedBuf, Object, ObjectHash};
use crate::object::tree::Tree;

#[derive(Debug, Clone)]
pub struct ObjectIo<Fs, Io>(FsIo<Fs, Io>)
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write;

impl<Fs, Io> Default for ObjectIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(FsIo::default())
    }
}

impl<Fs, Io> ObjectIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(fs: Fs) -> ObjectIo<Fs, Io> {
        Self(FsIo::new(fs))
    }

    pub fn read_to_tree(&self, object_hash: &ObjectHash) -> error::Result<Tree> {
        let obj = self.try_read_obj(object_hash)?;
        obj.deserialize()
    }

    pub fn try_read_obj(&self, object_hash: &ObjectHash) -> error::Result<Object> {
        self.read_obj(object_hash).and_then(|obj| {
            match obj {
                Some(obj) => Ok(obj),
                None => Err(error::Error::NotfoundObj(object_hash.clone())),
            }
        })
    }

    pub fn read_obj(&self, object_hash: &ObjectHash) -> error::Result<Option<Object>> {
        let Some(buf) = self
            .0
            .read_to_end(&format!("./.meltos/objects/{}", object_hash))?
            else {
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use meltos_util::compression::CompressionBuf;
    use meltos_util::compression::gz::Gz;

    use crate::file_system::{FileSystem, FsIo};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjectIo;
    use crate::io::atomic::workspace::WorkspaceIo;
    use crate::object::Object;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let mock = MockFileSystem::default();
        mock.create("test/hello.txt")
            .unwrap()
            .write_all(buf)
            .unwrap();

        let io = ObjectIo::new(mock.clone());
        let workspace = WorkspaceIo(FsIo::new(mock.clone()));
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
        let mock = MockFileSystem::default();
        let io = ObjectIo::new(mock.clone());
        let obj = Object::compress(b"hello world!".to_vec()).unwrap();
        io.write(&obj).unwrap();
        assert_eq!(io.read_obj(&obj.hash).unwrap(), Some(obj));
    }
}
