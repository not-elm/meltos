use crate::error;
use crate::file_system::FileSystem;
use crate::io::commit_obj::CommitObjIo;
use crate::object::commit::CommitHash;

#[derive(Debug, Clone)]
pub struct CommitHashIo<Fs>
    where
        Fs: FileSystem,
{
    commit_obj: CommitObjIo<Fs>,
}

impl<Fs> CommitHashIo<Fs>
    where
        Fs: FileSystem + Clone,
{
    #[inline(always)]
    pub fn new(fs: Fs) -> CommitHashIo<Fs> {
        Self {
            commit_obj: CommitObjIo::new(fs),
        }
    }
}

impl<Fs> CommitHashIo<Fs>
    where
        Fs: FileSystem,
{
    pub async fn read_all(
        &self,
        from: CommitHash,
        to: &Option<CommitHash>,
    ) -> error::Result<Vec<CommitHash>> {
        let mut hashes = Vec::new();
        self.read_obj(&mut hashes, from, to).await?;
        Ok(hashes)
    }

    #[async_recursion::async_recursion]
    async fn read_obj(
        &self,
        hashes: &mut Vec<CommitHash>,
        commit_hash: CommitHash,
        to: &Option<CommitHash>,
    ) -> error::Result<()> {
        let obj = self.commit_obj.read(&commit_hash).await?;
        hashes.push(commit_hash.clone());
        if !to.as_ref().is_some_and(|to| to == &commit_hash) {
            for parent_hash in obj.parents {
                self.read_obj(hashes, parent_hash, to).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn read_only_null() {
        let fs = MockFileSystem::default();
        let commit_hashes = crate::io::commit_hashes::CommitHashIo::new(fs.clone());
        let null_commit = init_owner_branch(fs.clone()).await;
        let hashes = commit_hashes.read_all(null_commit.clone(), &None).await.unwrap();
        assert_eq!(hashes, vec![null_commit]);
    }

    #[tokio::test]
    async fn read_with_parents() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let commit_hashes = crate::io::commit_hashes::CommitHashIo::new(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let commit0 = init_owner_branch(fs.clone()).await;

        fs.force_write("workspace/test.txt", b"hello");
        stage.execute(&branch, ".").await.unwrap();
        let commit1 = commit.execute(&branch, "commit").await.unwrap();

        fs.force_write("workspace/test2.txt", b"hello2");
        stage.execute(&branch, ".").await.unwrap();
        let commit2 = commit.execute(&branch, "commit").await.unwrap();

        let hashes = commit_hashes.read_all(commit2.clone(), &None).await.unwrap();
        assert_eq!(hashes, vec![commit2, commit1, commit0]);
    }
}
