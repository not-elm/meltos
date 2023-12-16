use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::commit::CommitHash;
use crate::object::{Decodable, Encodable, ObjHash};

#[derive(Debug, Clone)]
pub struct TraceIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
}

impl<Fs, Io> TraceIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> TraceIo<Fs, Io> {
        Self {
            io: FsIo::new(fs),
        }
    }

    #[inline]
    pub fn write(&self, commit_hash: &CommitHash, hash: &ObjHash) -> error::Result {
        let file_path = format!("./.meltos/traces/{commit_hash}");
        self.io.write(&file_path, &hash.encode()?)?;
        Ok(())
    }

    #[inline]
    pub fn read(&self, commit_hash: &CommitHash) -> error::Result<ObjHash> {
        let file_path = format!("./.meltos/traces/{commit_hash}");
        let buf = self
            .io
            .try_read(&file_path)
            .map_err(|_| error::Error::NotfoundTrace(commit_hash.clone()))?;
        ObjHash::decode(&buf)
    }
}
