use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;

#[derive(Debug, Clone)]
pub struct NewBranch<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
}


impl<Fs, Io> NewBranch<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub const fn new(fs: Fs) -> NewBranch<Fs, Io> {
        Self {
            head: HeadIo::new(fs),
        }
    }


    ///
    ///
    /// * copy `head file` from old branch
    /// * writes the `working` to new branch
    pub fn execute(&self, old: BranchName, new: BranchName) -> error::Result {
        let old_branch_head = self.head.try_read(&old)?;
        self.head.write(&new, &old_branch_head)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::operation::new_branch::NewBranch;
    use crate::tests::init_main_branch;

    #[test]
    fn copy_head_file() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());

        let new_branch = NewBranch::new(mock.clone());
        let head = HeadIo::new(mock);
        new_branch
            .execute(BranchName::main(), BranchName::from("second"))
            .unwrap();

        let head = head.try_read(&BranchName::from("second")).unwrap();
        assert_eq!(head, null_commit_hash);
    }
}
