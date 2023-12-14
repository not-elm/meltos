use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::ObjectHash;

#[derive(Debug, Clone)]
pub struct TraceIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
    file_path: String,
}

impl<Fs, Io> TraceIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> TraceIo<Fs, Io> {
        Self {
            file_path: format!("./.meltos/branches/{}/TRACE", branch_name),
            io: FsIo::new(fs),
        }
    }

    #[inline]
    pub fn write_hash(&self, hash: &ObjectHash) -> error::Result {
        self.io.write_all(&self.file_path, &hash.serialize_to_buf())?;
        Ok(())
    }

    #[inline]
    pub fn read_hash(&self) -> error::Result<Option<ObjectHash>> {
        let Some(buf) = self.io.read_to_end(&self.file_path)? else {
            return Ok(None);
        };
        Ok(Some(ObjectHash::from_serialized_buf(&buf)?))
    }

    #[inline]
    pub fn exists(&self) -> error::Result<bool> {
        Ok(self.read_hash()?.is_some())
    }
}
