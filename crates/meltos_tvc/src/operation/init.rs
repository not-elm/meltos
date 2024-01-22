use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;
use crate::object::commit::CommitHash;
use crate::operation::commit::Commit;
use crate::operation::stage::Stage;

#[derive(Debug, Clone)]
pub struct Init<Fs>
where
    Fs: FileSystem,
{
    commit: Commit<Fs>,
    working: WorkingIo<Fs>,
    stage: Stage<Fs>,
    fs: Fs,
}

impl<Fs> Init<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Init<Fs> {
        Self {
            commit: Commit::new(fs.clone()),
            working: WorkingIo::new(fs.clone()),
            stage: Stage::new(fs.clone()),
            fs,
        }
    }
}

impl<Fs> Init<Fs>
where
    Fs: FileSystem,
{
    /// Initialize the project.
    ///
    ///
    ///
    /// * create the `HEAD FILE`.
    /// * create `null commit`
    /// * create `head file` and write `null commit hash`
    /// * create `trace file` named `null commit hash`.
    /// * create `local commits file` and append `null commit hash`.
    /// * write `main` to the`WORKING`.
    pub fn execute(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        self.check_branch_not_initialized()?;
        self.working.write(branch_name)?;
        self.fs.create_dir("workspace")?;
        if self.stage.execute(branch_name, ".").is_ok() {
            self.commit.execute(branch_name, "INIT")
        } else {
            self.commit.execute_null_commit(branch_name)
        }
    }

    fn check_branch_not_initialized(&self) -> error::Result {
        if self.fs.all_files_in(".meltos")?.is_empty() {
            Ok(())
        } else {
            Err(error::Error::RepositoryAlreadyInitialized)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::encode::Encodable;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::atomic::object::ObjIo;
    use crate::object::commit::CommitHash;
    use crate::object::tree::TreeObj;
    use crate::object::{AsMeta, ObjHash};
    use crate::operation::commit::Commit;
    use crate::operation::init::Init;

    #[test]
    fn init() {
        let fs = MockFileSystem::default();
        let init = Init::new(fs.clone());
        init.execute(&BranchName::owner()).unwrap();
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        init.execute(&branch).unwrap();
        assert!(init.execute(&branch).is_err());
    }

    #[test]
    fn created_head_file() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());

        init.execute(&branch).unwrap();
        let head_commit_hash = read_head_commit_hash(fs.clone());
        let commit = Commit::new(fs.clone());
        let null_commit = commit.create_null_commit(TreeObj::default().as_meta().unwrap());
        assert_eq!(
            head_commit_hash,
            CommitHash(null_commit.as_meta().unwrap().hash)
        );
    }

    #[test]
    fn created_trace_file_named_null_commit_hash() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());

        init.execute(&branch).unwrap();

        let head_commit_hash = read_head_commit_hash(fs.clone());
        let trace_tree_hash = fs
            .read_file(&format!(".meltos/traces/{head_commit_hash}"))
            .unwrap();
        assert_eq!(
            trace_tree_hash,
            Some(TreeObj::default().as_meta().unwrap().hash.encode().unwrap())
        );
    }

    #[test]
    fn staged_workspace_files() {
        let fs = MockFileSystem::default();
        fs.force_write("workspace/src/test.rs", b"test");
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        init.execute(&branch).unwrap();
        assert!(ObjIo::new(fs)
            .read(&ObjHash::new(b"FILE\0test"))
            .unwrap()
            .is_some());
    }

    fn read_head_commit_hash(mock: MockFileSystem) -> CommitHash {
        let head = HeadIo::new(mock);
        head.try_read(&BranchName::owner()).unwrap()
    }
}
