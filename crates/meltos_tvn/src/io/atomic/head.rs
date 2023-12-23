use std::path::Path;

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
    fs: FsIo<Fs, Io>,
}

impl<Fs, Io> HeadIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub const fn new(fs: Fs) -> HeadIo<Fs, Io> {
        Self {
            fs: FsIo::new(fs),
        }
    }

    #[inline]
    pub fn write_remote(
        &self,
        branch_name: &BranchName,
        commit_hash: &CommitHash,
    ) -> error::Result {
        self.fs.write(
            &format!(".meltos/refs/remotes/{branch_name}"),
            &commit_hash.encode()?,
        )?;
        Ok(())
    }

    #[inline]
    pub fn write(&self, branch_name: &BranchName, commit_hash: &CommitHash) -> error::Result<()> {
        self.fs.write(
            &format!(".meltos/refs/heads/{branch_name}"),
            &commit_hash.encode()?,
        )?;
        Ok(())
    }

    #[inline]
    pub fn try_read_remote(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        self.read_remote(branch_name)?
            .ok_or_else(|| error::Error::NotfoundHead(branch_name.clone()))
    }

    #[inline]
    pub fn read_remote(&self, branch_name: &BranchName) -> error::Result<Option<CommitHash>> {
        self._read(".meltos/refs/remotes/", branch_name)
    }

    #[inline]
    pub fn try_read(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        self.read(branch_name)?
            .ok_or_else(|| error::Error::NotfoundHead(branch_name.clone()))
    }

    #[inline]
    pub fn read(&self, branch_name: &BranchName) -> error::Result<Option<CommitHash>> {
        self._read(".meltos/refs/heads/", branch_name)
    }

    pub fn read_all(&self) -> error::Result<Vec<(BranchName, CommitHash)>> {
        let files = self.fs.all_file_path(".meltos/refs/heads/")?;
        let mut branches = Vec::with_capacity(files.len());
        for path in files {
            let Some(file_name) = Path::new(&path).file_name().and_then(|name| name.to_str())
            else {
                continue;
            };
            let branch_name = BranchName::from(file_name);
            let commit_hash = self.try_read(&branch_name)?;
            branches.push((branch_name, commit_hash))
        }
        Ok(branches)
    }

    fn _read(&self, dir: &str, branch_name: &BranchName) -> error::Result<Option<CommitHash>> {
        let Some(buf) = self.fs.read(&format!("{dir}{branch_name}"))? else {
            return Ok(None);
        };
        Ok(Some(CommitHash(ObjHash::decode(&buf)?)))
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
