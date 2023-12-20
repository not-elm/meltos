use std::fmt::format;
use log::error;
use crate::branch::BranchName;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::bundle::Bundle;
use crate::io::commit_obj::CommitObjIo;

pub struct Merge<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read
{

    fs: FsIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
    commits_obj: CommitObjIo<Fs, Io>
}




impl<Fs, Io> Merge<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read
{
    pub fn execute(
        &self,
        from: BranchName,
        dist: BranchName
    ) -> crate::error::Result{
        let source_head = self.head.try_read(&from)?;
        let dist_head = self.head.try_read(&dist)?;
        let source_hashes = self.commits_obj.read_hashes(source_head.clone(), &None)?;
        let dist_hashes = self.commits_obj.read_hashes(dist_head.clone(), &None)?;
        if source_hashes.contains(&dist_head){
            self.head.write(&dist, &source_head)?;
            return Ok(());
        }
        if dist_hashes.contains(&source_head){
            return Ok(());
        }

        Ok(())
    }


}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MergeConfig{

}
