use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::work_branch::WorkingIo;

#[derive(Debug, Clone)]
pub struct NewBranch<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    working: WorkingIo<Fs, Io>,
    fs: FsIo<Fs, Io>,
}


impl<Fs, Io> NewBranch<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> NewBranch<Fs, Io> {
        Self {
            fs: FsIo::new(fs.clone()),
            working: WorkingIo::new(fs),
        }
    }


    ///
    ///
    /// * copy `head file` from old branch
    /// * writes the `working` to new branch
    pub fn execute(&self, old: BranchName, new: BranchName) -> error::Result {
        let old_branch_head = HeadIo::new(old, self.fs.clone()).read()?;
        self.working.write(&new)?;
        HeadIo::new(new, self.fs.clone()).write(old_branch_head)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::atomic::work_branch::WorkingIo;
    use crate::operation::new_branch::NewBranch;
    use crate::tests::init_main_branch;

    #[test]
    fn update_working_file() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let new_branch = NewBranch::new(mock.clone());
        let working = WorkingIo::new(mock);
        new_branch
            .execute(BranchName::main(), BranchName::from("second"))
            .unwrap();
        let branch_name = working.read().unwrap();
        assert_eq!(branch_name, BranchName::from("second"));
    }

    #[test]
    fn copy_head_file() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());

        let new_branch = NewBranch::new(mock.clone());
        let head = HeadIo::new(BranchName::from("second"), mock);
        new_branch
            .execute(BranchName::main(), BranchName::from("second"))
            .unwrap();

        let head = head.read().unwrap();
        assert_eq!(head, null_commit_hash);
    }
}
