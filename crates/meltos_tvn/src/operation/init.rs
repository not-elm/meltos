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
    branch_name: BranchName,
}

impl<Fs> Init<Fs>
    where
        Fs: FileSystem + Clone,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Init<Fs> {
        Self {
            commit: Commit::new(branch_name.clone(), fs.clone()),
            working: WorkingIo::new(fs.clone()),
            stage: Stage::new(branch_name.clone(), fs.clone()),
            fs,
            branch_name,
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
    pub fn execute(&self) -> error::Result<CommitHash> {
        self.check_branch_not_initialized()?;
        self.working.write(&self.branch_name)?;
        if self.stage.execute(".").is_ok() {
            self.commit.execute("INIT")
        } else {
            self.commit.execute_null_commit()
        }
    }

    fn check_branch_not_initialized(&self) -> error::Result {
        if self.fs.all_file_path(".meltos")?.is_empty() {
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
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::atomic::object::ObjIo;
    use crate::object::{AsMeta, ObjHash};
    use crate::object::commit::CommitHash;
    use crate::object::tree::TreeObj;
    use crate::operation::commit::Commit;
    use crate::operation::init;
    use crate::operation::init::Init;

    #[test]
    fn init() {
        let mock = MockFileSystem::default();
        let init = Init::new(BranchName::owner(), mock.clone());
        init.execute().unwrap();
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let mock = MockFileSystem::default();
        let init = Init::new(BranchName::owner(), mock.clone());
        init.execute().unwrap();
        assert!(init.execute().is_err());
    }

    #[test]
    fn created_head_file() {
        let mock = MockFileSystem::default();
        let branch = BranchName::owner();
        let init = init::Init::new(branch.clone(), mock.clone());

        init.execute().unwrap();
        let head_commit_hash = read_head_commit_hash(mock.clone());
        let commit = Commit::new(BranchName::owner(), mock.clone());
        let null_commit = commit.create_null_commit(TreeObj::default().as_meta().unwrap());
        assert_eq!(
            head_commit_hash,
            CommitHash(null_commit.as_meta().unwrap().hash)
        );
    }

    #[test]
    fn created_trace_file_named_null_commit_hash() {
        let mock = MockFileSystem::default();
        let branch = BranchName::owner();
        let init = init::Init::new(branch.clone(), mock.clone());

        init.execute().unwrap();

        let head_commit_hash = read_head_commit_hash(mock.clone());
        let trace_tree_hash = mock
            .read(&format!(".meltos/traces/{head_commit_hash}"))
            .unwrap();
        assert_eq!(
            trace_tree_hash,
            Some(TreeObj::default().as_meta().unwrap().hash.encode().unwrap())
        );
    }

    #[test]
    fn staged_workspace_files() {
        let mock = MockFileSystem::default();
        mock.force_write("./workspace/src/test.rs", b"test");
        let init = Init::new(BranchName::owner(), mock.clone());
        init.execute().unwrap();
        assert!(ObjIo::new(mock)
            .read(&ObjHash::new(b"FILE\0test"))
            .unwrap()
            .is_some());
    }

    fn read_head_commit_hash(mock: MockFileSystem) -> CommitHash {
        let head = HeadIo::new(mock);
        head.try_read(&BranchName::owner()).unwrap()
    }
}
