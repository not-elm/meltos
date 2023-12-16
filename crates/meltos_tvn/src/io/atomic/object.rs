use std::io;

use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::{CompressedBuf, ObjMeta, ObjHash, Decodable};
use crate::object::commit::{CommitHash, CommitObj};
use crate::object::tree::TreeObj;

#[derive(Debug, Clone)]
pub struct ObjIo<Fs, Io>(FsIo<Fs, Io>)
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write;

impl<Fs, Io> Default for ObjIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(FsIo::default())
    }
}

impl<Fs, Io> ObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(fs: Fs) -> ObjIo<Fs, Io> {
        Self(FsIo::new(fs))
    }


    pub fn read_to_commit(&self, object_hash: &CommitHash) -> error::Result<CommitObj> {
        let obj = self.try_read_obj(&object_hash.0)?;
        CommitObj::try_from(obj)
    }


    pub fn read_to_tree(&self, object_hash: &ObjHash) -> error::Result<TreeObj> {
        let meta = self.try_read_obj(object_hash)?;
        TreeObj::decode(&meta.buf)
    }

    pub fn try_read_obj(&self, object_hash: &ObjHash) -> error::Result<ObjMeta> {
        self.read_obj(object_hash).and_then(|obj| {
            match obj {
                Some(obj) => Ok(obj),
                None => Err(error::Error::NotfoundObj(object_hash.clone())),
            }
        })
    }

    pub fn read_obj(&self, object_hash: &ObjHash) -> error::Result<Option<ObjMeta>> {
        let Some(buf) = self.read(object_hash)?
            else {
                return Ok(None);
            };

        Ok(Some(ObjMeta::expand(buf)?))
    }


    pub fn read(&self, object_hash: &ObjHash) -> error::Result<Option<CompressedBuf>> {
        let Some(buf) = self
            .0
            .read(&format!("./.meltos/objects/{}", object_hash))?
            else {
                return Ok(None);
            };

        Ok(Some(CompressedBuf(buf)))
    }

    pub fn write(&self, obj: &ObjMeta) -> io::Result<()> {
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
    use crate::io::atomic::object::ObjIo;
    use crate::io::atomic::workspace::WorkspaceIo;
    use crate::object::ObjMeta;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let mock = MockFileSystem::default();
        mock.create("test/hello.txt")
            .unwrap()
            .write_all(buf)
            .unwrap();

        let io = ObjIo::new(mock.clone());
        let workspace = WorkspaceIo(FsIo::new(mock.clone()));
        let mut objs = workspace.convert_to_objs("test/hello.txt").unwrap();
        let meta = objs.next().unwrap().unwrap();
        io.write(&meta.obj).unwrap();

        let hello_buf = mock
            .try_read(&format!(
                "./.meltos/objects/{}",
                meltos_util::hash::hash(buf)
            ))
            .unwrap();
        assert_eq!(hello_buf, Gz.encode(buf).unwrap());
    }

    #[test]
    fn read_obj() {
        let mock = MockFileSystem::default();
        let io = ObjIo::new(mock.clone());
        let obj = ObjMeta::compress(b"hello world!".to_vec()).unwrap();
        io.write(&obj).unwrap();
        assert_eq!(io.read_obj(&obj.hash).unwrap(), Some(obj));
    }
}
