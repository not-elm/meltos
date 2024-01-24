use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;

#[derive(Debug, Clone)]
pub struct NewBranch<Fs>
where
    Fs: FileSystem,
{
    head: HeadIo<Fs>,
}

impl<Fs> NewBranch<Fs>
where
    Fs: FileSystem,
{
    #[inline]
    pub const fn new(fs: Fs) -> NewBranch<Fs> {
        Self {
            head: HeadIo::new(fs),
        }
    }

    ///
    ///
    /// * copy `head file` from old branch
    /// * writes the `working` to new branch
    pub async fn execute(&self, old: BranchName, new: BranchName) -> error::Result {
        let old_branch_head = self.head.try_read(&old).await?;
        self.head.write(&new, &old_branch_head).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::operation::new_branch::NewBranch;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn copy_head_file() {
        let fs = MockFileSystem::default();
        let null_commit_hash = init_owner_branch(fs.clone()).await;

        let new_branch = NewBranch::new(fs.clone());
        let head = HeadIo::new(fs);
        new_branch
            .execute(BranchName::owner(), BranchName::from("second"))
            .await
            .unwrap();

        let head = head.try_read(&BranchName::from("second")).await.unwrap();
        assert_eq!(head, null_commit_hash);
    }
}
