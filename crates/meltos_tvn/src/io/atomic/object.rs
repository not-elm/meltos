use std::io;
use std::path::Path;

use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::io::bundle::BundleObject;
use crate::object::commit::{CommitHash, CommitObj};
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;
use crate::object::{AsMeta, CompressedBuf, Obj, ObjHash};

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

    #[inline]
    pub fn read_to_commit(&self, object_hash: &CommitHash) -> error::Result<CommitObj> {
        let obj = self.try_read_obj(&object_hash.0)?;
        obj.commit()
    }

    #[inline]
    pub fn read_to_file(&self, object_hash: &ObjHash) -> error::Result<FileObj> {
        let obj = self.try_read_obj(object_hash)?;
        obj.file()
    }

    #[inline]
    pub fn read_to_tree(&self, object_hash: &ObjHash) -> error::Result<TreeObj> {
        let obj = self.try_read_obj(object_hash)?;
        obj.tree()
    }

    pub fn try_read_obj(&self, object_hash: &ObjHash) -> error::Result<Obj> {
        self.read_obj(object_hash).and_then(|obj| {
            match obj {
                Some(obj) => Ok(obj),
                None => Err(error::Error::NotfoundObj(object_hash.clone())),
            }
        })
    }

    pub fn read_obj(&self, object_hash: &ObjHash) -> error::Result<Option<Obj>> {
        let Some(buf) = self.read(object_hash)? else {
            return Ok(None);
        };

        Ok(Some(Obj::expand(&buf)?))
    }

    pub fn read_all(&self) -> error::Result<Vec<BundleObject>> {
        let files = self.0.all_file_path("./.meltos/objects/")?;
        let mut objs = Vec::with_capacity(files.len());
        for path in files {
            let buf = self.0.try_read(&path)?;
            let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
            objs.push(BundleObject {
                hash: ObjHash(file_name.to_string()),
                compressed_buf: CompressedBuf(buf),
            });
        }
        Ok(objs)
    }

    pub fn read(&self, object_hash: &ObjHash) -> error::Result<Option<CompressedBuf>> {
        let Some(buf) = self.0.read(&format!("./.meltos/objects/{}", object_hash))? else {
            return Ok(None);
        };

        Ok(Some(CompressedBuf(buf)))
    }

    pub fn write_obj(&self, obj: &impl AsMeta) -> error::Result<()> {
        let obj = obj.as_meta()?;
        self.write(&obj.hash, &obj.compressed_buf)
    }

    pub fn write_all(&self, objs: &[BundleObject]) -> error::Result {
        for BundleObject {
            hash,
            compressed_buf,
        } in objs
        {
            self.write(hash, compressed_buf)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write(&self, hash: &ObjHash, compressed_buf: &CompressedBuf) -> error::Result {
        self.0
            .create(&format!("./.meltos/objects/{}", hash))?
            .write_all(compressed_buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use meltos_util::compression::gz::Gz;
    use meltos_util::compression::CompressionBuf;

    use crate::encode::Decodable;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::workspace::WorkspaceIo;
    use crate::object::file::FileObj;
    use crate::object::{AsMeta, Obj, ObjMeta};

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let mock = MockFileSystem::default();
        mock.create("test/hello.txt")
            .unwrap()
            .write_all(buf)
            .unwrap();

        let io = ObjIo::new(mock.clone());
        let workspace = WorkspaceIo::new(mock.clone());
        let mut objs = workspace.convert_to_objs("test/hello.txt").unwrap();
        let (_, file_obj) = objs.next().unwrap().unwrap();
        io.write_obj(&Obj::File(file_obj)).unwrap();

        let hello_buf = mock
            .try_read(&format!(
                "./.meltos/objects/{}",
                meltos_util::hash::hash(b"FILE\0hello world!")
            ))
            .unwrap();
        assert_eq!(hello_buf, Gz.zip(b"FILE\0hello world!").unwrap());
    }

    #[test]
    fn read_obj() {
        let mock = MockFileSystem::default();
        let io = ObjIo::new(mock.clone());
        let obj = ObjMeta::compress(b"FILE\0hello world!".to_vec()).unwrap();
        io.write_obj(&FileObj::decode(b"FILE\0hello world!").unwrap())
            .unwrap();
        assert_eq!(io.try_read_obj(&obj.hash).unwrap().as_meta().unwrap(), obj);
    }
}
