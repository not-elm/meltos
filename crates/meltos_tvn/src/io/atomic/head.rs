use meltos_util::impl_string_new_type;

use crate::branch::BranchName;
use crate::encode::{Decodable, Encodable};
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::commit::CommitHash;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct HeadIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
}


impl<Fs, Io> HeadIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub const fn new(fs: Fs) -> HeadIo<Fs, Io> {
        Self {
            io: FsIo::new(fs),
        }
    }

    pub fn write(&self, branch_name: &BranchName, commit_hash: &CommitHash) -> error::Result<()> {
        self.io.write(
            &format!(".meltos/refs/heads/{branch_name}"),
            &commit_hash.encode()?,
        )?;
        Ok(())
    }

    pub fn read(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        let buf = self
            .io
            .try_read(&format!(".meltos/refs/heads/{branch_name}"))
            .map_err(|_| error::Error::NotfoundHead)?;
        Ok(CommitHash(ObjHash::decode(&buf)?))
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct CommitText(pub String);
impl_string_new_type!(CommitText);

impl Encodable for CommitText {
    #[inline]
    fn encode(&self) -> error::Result<Vec<u8>> {
        Ok(self.0.as_bytes().to_vec())
    }
}


impl Decodable for CommitText {
    fn decode(buf: &[u8]) -> error::Result<Self> {
        Ok(Self(String::from_utf8(buf.to_vec()).unwrap()))
    }
}
