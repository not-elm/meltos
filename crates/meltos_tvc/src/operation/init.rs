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
    pub async fn execute(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        self.check_branch_not_initialized().await?;
        self.working.write(branch_name).await?;
        self.fs.create_dir("workspace").await?;
        if self.stage.execute(branch_name, ".").await.is_ok() {
            self.commit.execute(branch_name, "INIT").await
        } else {
            self.commit.execute_null_commit(branch_name).await
        }
    }

    async fn check_branch_not_initialized(&self) -> error::Result {
        if self.fs.all_files_in(".meltos_core").await?.is_empty() {
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
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::atomic::object::ObjIo;
    use crate::object::commit::CommitHash;
    use crate::object::tree::TreeObj;
    use crate::object::{AsMeta, ObjHash};
    use crate::operation::commit::Commit;
    use crate::operation::init::Init;

    #[tokio::test]
    async fn init() {
        let fs = MemoryFileSystem::default();
        let init = Init::new(fs.clone());
        init.execute(&BranchName::owner()).await.unwrap();
    }

    #[tokio::test]
    async fn failed_init_if_has_been_initialized() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        init.execute(&branch).await.unwrap();
        assert!(init.execute(&branch).await.is_err());
    }

    #[tokio::test]
    async fn created_head_file() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());

        init.execute(&branch).await.unwrap();
        let head_commit_hash = read_head_commit_hash(fs.clone()).await;
        let commit = Commit::new(fs.clone());
        let null_commit = commit.create_null_commit(TreeObj::default().as_meta().unwrap());
        assert_eq!(
            head_commit_hash,
            CommitHash(null_commit.as_meta().unwrap().hash)
        );
    }

    #[tokio::test]
    async fn created_trace_file_named_null_commit_hash() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());

        init.execute(&branch).await.unwrap();

        let head_commit_hash = read_head_commit_hash(fs.clone()).await;
        let trace_tree_hash = fs
            .read_file(&format!(".meltos_core/traces/{head_commit_hash}"))
            .await
            .unwrap();
        assert_eq!(
            trace_tree_hash,
            Some(TreeObj::default().as_meta().unwrap().hash.encode().unwrap())
        );
    }

    #[tokio::test]
    async fn staged_workspace_files() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("workspace/src/test.rs", b"test");
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        init.execute(&branch).await.unwrap();
        assert!(ObjIo::new(fs)
            .read(&ObjHash::new(b"FILE\0test"))
            .await
            .unwrap()
            .is_some());
    }

    async fn read_head_commit_hash(mock: MemoryFileSystem) -> CommitHash {
        let head = HeadIo::new(mock);
        head.try_read(&BranchName::owner()).await.unwrap()
    }
}
