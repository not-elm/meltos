use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::work_branch::WorkingIo;
use crate::object::commit::CommitHash;
use crate::operation::commit::Commit;
use crate::operation::stage::Stage;

#[derive(Debug, Clone)]
pub struct Init<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    commit: Commit<Fs, Io>,
    working: WorkingIo<Fs, Io>,
    stage: Stage<Fs, Io>,
    fs: FsIo<Fs, Io>,
}


impl<Fs, Io> Init<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Init<Fs, Io> {
        Self {
            commit: Commit::new(branch_name.clone(), fs.clone()),
            working: WorkingIo::new(fs.clone()),
            stage: Stage::new(branch_name, fs.clone()),
            fs: FsIo::new(fs.clone()),
        }
    }
}


impl<Fs, Io> Init<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
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
        self.working.write(&BranchName::main())?;
        if self.stage.execute(".").is_ok() {
            self.commit.execute("INIT")
        } else {
            self.commit.execute_null_commit()
        }
    }


    fn check_branch_not_initialized(&self) -> error::Result {
        println!("{:?}", self.fs.all_file_path("./.meltos")?);
        if self.fs.all_file_path("./.meltos")?.is_empty() {
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
    use crate::operation::init;
    use crate::operation::init::Init;

    #[test]
    fn init() {
        let mock = MockFileSystem::default();
        let init = Init::new(BranchName::main(), mock.clone());
        init.execute().unwrap();
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let mock = MockFileSystem::default();
        let init = Init::new(BranchName::main(), mock.clone());
        init.execute().unwrap();
        assert!(init.execute().is_err());
    }


    #[test]
    fn created_head_file() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let init = init::Init::new(branch.clone(), mock.clone());

        init.execute().unwrap();
        let head_commit_hash = read_head_commit_hash(mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());
        let null_commit = commit.create_null_commit(TreeObj::default().as_meta().unwrap());
        assert_eq!(
            head_commit_hash,
            CommitHash(null_commit.as_meta().unwrap().hash)
        );
    }


    #[test]
    fn created_trace_file_named_null_commit_hash() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let init = init::Init::new(branch.clone(), mock.clone());

        init.execute().unwrap();

        let head_commit_hash = read_head_commit_hash(mock.clone());
        let trace_tree_hash = mock
            .read(&format!("./.meltos/traces/{head_commit_hash}"))
            .unwrap();
        assert_eq!(
            trace_tree_hash,
            Some(TreeObj::default().as_meta().unwrap().hash.encode().unwrap())
        );
    }

    #[test]
    fn staged_workspace_files() {
        let mock = MockFileSystem::default();
        mock.force_write("./src/test.rs", b"test");
        let init = Init::new(BranchName::main(), mock.clone());
        init.execute().unwrap();
        assert!(ObjIo::new(mock)
            .read(&ObjHash::new(b"FILE\0test"))
            .unwrap()
            .is_some());
    }

    fn read_head_commit_hash(mock: MockFileSystem) -> CommitHash {
        let head = HeadIo::new(mock);
        head.try_read(&BranchName::main()).unwrap()
    }
}
