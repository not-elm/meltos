use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::ObjHash;

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
    pub fn write(&self, hash: &ObjHash) -> error::Result {
        self.io.write(&self.file_path, &hash.serialize_to_buf())?;
        Ok(())
    }

    #[inline]
    pub fn read_hash(&self) -> error::Result<Option<ObjHash>> {
        let Some(buf) = self.io.read(&self.file_path)? else {
            return Ok(None);
        };
        Ok(Some(ObjHash::from_serialized_buf(&buf)?))
    }

    #[inline]
    pub fn exists(&self) -> error::Result<bool> {
        Ok(self.read_hash()?.is_some())
    }
}
