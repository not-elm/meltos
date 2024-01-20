use std::path::Path;

use crate::error;
use crate::file_system::FileSystem;
use crate::io::bundle::BundleObject;
use crate::object::{AsMeta, CompressedBuf, Obj, ObjHash};
use crate::object::commit::{CommitHash, CommitObj};
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;

#[derive(Debug, Clone, Default)]
pub struct ObjIo<Fs>(Fs)
    where
        Fs: FileSystem;

impl<Fs> ObjIo<Fs>
    where
        Fs: FileSystem,
{
    #[inline]
    pub const fn new(fs: Fs) -> ObjIo<Fs> {
        Self(fs)
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
    pub fn try_read_to_file(&self, object_hash: &ObjHash) -> error::Result<Option<FileObj>> {
        let Some(obj) = self.read_obj(object_hash)?
            else {
                return Ok(None);
            };
        Ok(Some(obj.file()?))
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


    #[inline(always)]
    pub fn total_objs_size(&self) -> error::Result<usize> {
        Ok(self
            .read_all()?
            .iter()
            .map(|o| o.compressed_buf.0.len())
            .sum())
    }


    pub fn read_all(&self) -> error::Result<Vec<BundleObject>> {
        let files = self.0.all_files_in(".meltos/objects")?;
        let mut objs = Vec::with_capacity(files.len());
        for path in files {
            let buf = self.0.try_read_file(&path)?;
            let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
            objs.push(BundleObject {
                hash: ObjHash(file_name.to_string()),
                compressed_buf: CompressedBuf(buf),
            });
        }
        Ok(objs)
    }

    pub fn read(&self, object_hash: &ObjHash) -> error::Result<Option<CompressedBuf>> {
        let Some(buf) = self
            .0
            .read_file(&format!(".meltos/objects/{}", object_hash))?
            else {
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
            .write_file(&format!(".meltos/objects/{}", hash), compressed_buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meltos_util::compression::CompressionBuf;
    use meltos_util::compression::gz::Gz;

    use crate::encode::Decodable;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::workspace::WorkspaceIo;
    use crate::object::{AsMeta, Obj, ObjMeta};
    use crate::object::file::FileObj;

    #[test]
    fn write_object_file() {
        let buf = b"hello world!";
        let fs = MockFileSystem::default();
        fs.force_write("workspace/test/hello.txt", buf);

        let io = ObjIo::new(fs.clone());
        let workspace = WorkspaceIo::new(fs.clone());
        let mut objs = workspace.convert_to_objs("test/hello.txt").unwrap();
        let (_, file_obj) = objs.next().unwrap().unwrap();
        io.write_obj(&Obj::File(file_obj)).unwrap();

        let hello_buf = fs
            .try_read_file(&format!(
                ".meltos/objects/{}",
                meltos_util::hash::hash(b"FILE\0hello world!")
            ))
            .unwrap();
        assert_eq!(hello_buf, Gz.zip(b"FILE\0hello world!").unwrap());
    }

    #[test]
    fn read_obj() {
        let fs = MockFileSystem::default();
        let io = ObjIo::new(fs.clone());
        let obj = ObjMeta::compress(b"FILE\0hello world!".to_vec()).unwrap();
        io.write_obj(&FileObj::decode(b"FILE\0hello world!").unwrap())
            .unwrap();
        assert_eq!(io.try_read_obj(&obj.hash).unwrap().as_meta().unwrap(), obj);
    }
}
