use std::path::Path;

use crate::error;
use crate::file_system::FileSystem;
use crate::io::bundle::BundleObject;
use crate::object::commit::{CommitHash, CommitObj};
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;
use crate::object::{AsMeta, CompressedBuf, Obj, ObjHash};

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
    pub async fn read_to_commit(&self, object_hash: &CommitHash) -> error::Result<CommitObj> {
        let obj = self.try_read_obj(&object_hash.0).await?;
        obj.commit()
    }

    #[inline]
    pub async fn read_to_file(&self, object_hash: &ObjHash) -> error::Result<FileObj> {
        let obj = self.try_read_obj(object_hash).await?;
        obj.file()
    }

    #[inline]
    pub async fn try_read_to_file(&self, object_hash: &ObjHash) -> error::Result<Option<FileObj>> {
        let Some(obj) = self.read_obj(object_hash).await? else {
            return Ok(None);
        };
        Ok(Some(obj.file()?))
    }

    #[inline]
    pub async fn read_to_tree(&self, object_hash: &ObjHash) -> error::Result<TreeObj> {
        let obj = self.try_read_obj(object_hash).await?;
        obj.tree()
    }

    pub async fn try_read_obj(&self, object_hash: &ObjHash) -> error::Result<Obj> {
        self.read_obj(object_hash).await.and_then(|obj| {
            match obj {
                Some(obj) => Ok(obj),
                None => Err(error::Error::NotfoundObj(object_hash.clone())),
            }
        })
    }

    pub async fn read_obj(&self, object_hash: &ObjHash) -> error::Result<Option<Obj>> {
        let Some(buf) = self.read(object_hash).await? else {
            return Ok(None);
        };

        Ok(Some(Obj::expand(&buf)?))
    }

    #[inline(always)]
    pub async fn total_objs_size(&self) -> error::Result<usize> {
        Ok(self
            .read_all()
            .await?
            .iter()
            .map(|o| o.compressed_buf.0.len())
            .sum())
    }

    pub async fn read_all(&self) -> error::Result<Vec<BundleObject>> {
        let files = self.0.all_files_in(".meltos/objects").await?;
        let mut objs = Vec::with_capacity(files.len());
        for path in files {
            let buf = self.0.try_read_file(&path).await?;
            let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();
            objs.push(BundleObject {
                hash: ObjHash(file_name.to_string()),
                compressed_buf: CompressedBuf(buf),
            });
        }
        Ok(objs)
    }

    pub async fn read(&self, object_hash: &ObjHash) -> error::Result<Option<CompressedBuf>> {
        let Some(buf) = self
            .0
            .read_file(&format!(".meltos/objects/{}", object_hash))
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(CompressedBuf(buf)))
    }

    pub async fn write_obj(&self, obj: &impl AsMeta) -> error::Result<()> {
        let obj = obj.as_meta()?;
        self.write(&obj.hash, &obj.compressed_buf).await
    }

    pub async fn write_all(&self, objs: &[BundleObject]) -> error::Result {
        for BundleObject {
            hash,
            compressed_buf,
        } in objs
        {
            self.write(hash, compressed_buf).await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn write(&self, hash: &ObjHash, compressed_buf: &CompressedBuf) -> error::Result {
        self.0
            .write_file(&format!(".meltos/objects/{}", hash), compressed_buf)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meltos_util::compression::gz::Gz;
    use meltos_util::compression::CompressionBuf;

    use crate::encode::Decodable;
    use crate::error;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::workspace::WorkspaceIo;
    use crate::object::file::FileObj;
    use crate::object::{AsMeta, Obj, ObjMeta};

    #[tokio::test]
    async fn write_object_file() -> error::Result {
        let buf = b"hello world!";
        let fs = MemoryFileSystem::default();
        fs.write_sync("workspace/test/hello.txt", buf);

        let io = ObjIo::new(fs.clone());
        let workspace = WorkspaceIo::new(fs.clone());
        let mut objs = workspace.convert_to_objs("test/hello.txt").await.unwrap();
        let (_, file_obj) = objs.next().await.unwrap().unwrap();
        io.write_obj(&Obj::File(file_obj)).await?;

        let hello_buf = fs
            .try_read_file(&format!(
                ".meltos/objects/{}",
                meltos_util::hash::hash(b"FILE\0hello world!")
            ))
            .await?;
        assert_eq!(hello_buf, Gz.zip(b"FILE\0hello world!").unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn read_obj() {
        let fs = MemoryFileSystem::default();
        let io = ObjIo::new(fs.clone());
        let obj = ObjMeta::compress(b"FILE\0hello world!".to_vec()).unwrap();
        io.write_obj(&FileObj::decode(b"FILE\0hello world!").unwrap())
            .await
            .unwrap();
        assert_eq!(
            io.try_read_obj(&obj.hash).await.unwrap().as_meta().unwrap(),
            obj
        );
    }
}
