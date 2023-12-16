use std::io;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::local_commits::LocalCommitsObj;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct LocalCommitsIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write
{
    fs: FsIo<Fs, Io>,
    file_path: String,
}


impl<Fs, Io> LocalCommitsIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    #[inline]
    pub fn new(branch_name: BranchName, fs: Fs) -> LocalCommitsIo<Fs, Io> {
        Self {
            fs: FsIo::new(fs),
            file_path: format!("./.meltos/branches/{branch_name}/LOCAL"),
        }
    }

    pub fn write(&self, local_commits: &LocalCommitsObj) -> error::Result {
        self.fs.write(&self.file_path, &local_commits.to_buf())?;
        Ok(())
    }
    

    pub fn append(&self, commit_hash: ObjHash) -> error::Result {
        let mut local_commits = self.read()?.unwrap_or_default();
        local_commits.push(commit_hash);
        self.write(&local_commits)
    }


    pub fn read(&self) -> error::Result<Option<LocalCommitsObj>> {
        let Some(buf) = self.fs.read(&self.file_path)?
            else {
                return Ok(None);
            };

        Ok(Some(LocalCommitsObj::new(buf)))
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::local_commits::LocalCommitsIo;
    use crate::object::local_commits::LocalCommitsObj;
    use crate::object::ObjHash;

    #[test]
    fn append_one_commit() {
        let hash = ObjHash::new(b"commit hash");
        let io = LocalCommitsIo::new(BranchName::main(), MockFileSystem::default());
        io.append(hash.clone()).unwrap();
        let local_commits = io.read().unwrap().unwrap();
        assert_eq!(local_commits, LocalCommitsObj(vec![hash]));
    }
}