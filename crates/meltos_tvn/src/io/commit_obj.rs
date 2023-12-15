use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::{CommitText, HeadIo};
use crate::io::atomic::object::ObjectIo;
use crate::object::commit::CommitObj;
use crate::object::ObjHash;

pub struct CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
    object: ObjectIo<Fs, Io>,
}


impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, fs: Fs) -> CommitObjIo<Fs, Io> {
        CommitObjIo {
            head: HeadIo::new(branch_name.clone(), fs.clone()),
            object: ObjectIo::new(fs.clone()),
        }
    }
}

impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn read(&self) -> error::Result<Option<CommitObj>> {
        let Some(hash) = self.head.head_commit_hash()?
            else {
                return Ok(None);
            };
        let commit_obj = self.object.try_read_obj(&hash)?;
        Ok(Some(CommitObj::try_from(commit_obj)?))
    }

    pub fn create(
        &self,
        commit_text: impl Into<CommitText>,
        staging_hash: ObjHash,
    ) -> error::Result<CommitObj> {
        let head_commit = self.head.head_commit_hash()?;
        Ok(CommitObj {
            parent: head_commit,
            text: commit_text.into(),
            stage: staging_hash,
        })
    }
}