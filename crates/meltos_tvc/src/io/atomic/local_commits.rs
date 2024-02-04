use crate::branch::BranchName;
use crate::encode::{Decodable, Encodable};
use crate::error;
use crate::file_system::FileSystem;
use crate::object::commit::CommitHash;
use crate::object::local_commits::LocalCommitsObj;

#[derive(Debug, Clone)]
pub struct LocalCommitsIo<Fs>
where
    Fs: FileSystem,
{
    fs: Fs,
}

impl<Fs> LocalCommitsIo<Fs>
where
    Fs: FileSystem,
{
    #[inline(always)]
    pub const fn new(fs: Fs) -> LocalCommitsIo<Fs> {
        Self {
            fs,
        }
    }

    pub async fn write(
        &self,
        local_commits: &LocalCommitsObj,
        branch_name: &BranchName,
    ) -> error::Result {
        self.fs
            .write_file(&self.file_path(branch_name), &local_commits.encode()?)
            .await?;
        Ok(())
    }

    pub async fn append(&self, commit_hash: CommitHash, branch_name: &BranchName) -> error::Result {
        let mut local_commits = self.read(branch_name).await?.unwrap_or_default();
        local_commits.push(commit_hash);
        self.write(&local_commits, branch_name).await
    }

    pub async fn try_read(&self, branch_name: &BranchName) -> error::Result<LocalCommitsObj> {
        let Some(local_commits) = self.read(branch_name).await? else {
            return Err(error::Error::NotfoundLocalCommits);
        };
        Ok(local_commits)
    }

    pub async fn read(&self, branch_name: &BranchName) -> error::Result<Option<LocalCommitsObj>> {
        let Some(buf) = self.fs.read_file(&self.file_path(branch_name)).await? else {
            return Ok(None);
        };

        Ok(Some(LocalCommitsObj::decode(&buf)?))
    }

    #[inline(always)]
    fn file_path(&self, branch_name: &BranchName) -> String {
        format!(".meltos/branches/{branch_name}/LOCAL")
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::io::atomic::local_commits::LocalCommitsIo;
    use crate::object::commit::CommitHash;
    use crate::object::local_commits::LocalCommitsObj;
    use crate::object::ObjHash;

    #[tokio::test]
    async fn append_one_commit() {
        let hash = CommitHash(ObjHash::new(b"commit hash"));
        let branch_name = BranchName::owner();
        let io = LocalCommitsIo::new(MemoryFileSystem::default());
        io.append(hash.clone(), &branch_name).await.unwrap();
        let local_commits = io.read(&branch_name).await.unwrap().unwrap();
        assert_eq!(local_commits, LocalCommitsObj(vec![hash]));
    }
}
