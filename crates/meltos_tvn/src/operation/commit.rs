use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::{CommitText, HeadIo};
use crate::io::atomic::local_commits::LocalCommitsIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::commit_obj::CommitObjIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::commit::{CommitHash, CommitObj};
use crate::object::tree::TreeObj;
use crate::object::{AsMeta, ObjMeta};

#[derive(Debug, Clone)]
pub struct Commit<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    commit_obj: CommitObjIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    staging: StagingIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>,
    local_commits: LocalCommitsIo<Fs, Io>,
}

impl<Fs, Io> Commit<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Commit<Fs, Io> {
        Self {
            commit_obj: CommitObjIo::new(branch_name.clone(), fs.clone()),
            head: HeadIo::new(branch_name.clone(), fs.clone()),
            object: ObjIo::new(fs.clone()),
            staging: StagingIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name, fs),
        }
    }
}


impl<Fs, Io> Commit<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn execute(&self, commit_text: impl Into<CommitText>) -> error::Result<CommitHash> {
        let Some(stage_tree) = self.staging.read()? else {
            return Err(error::Error::NotfoundStages);
        };
        self.staging.reset()?;
        let stage_obj = stage_tree.as_meta()?;
        self.object.write(&stage_obj)?;

        let commit = self.commit_obj.create(commit_text, stage_obj.hash)?;
        let head_commit_hash = self.commit(commit)?;
        self.update_trace(stage_tree, &head_commit_hash)?;
        Ok(head_commit_hash)
    }


    ///
    /// * create `null commit`
    /// * create `head file` and write `null commit hash`
    /// * create `trace file` named `null commit hash`.
    /// * create `local commits file` and append `null commit hash`
    pub fn execute_null_commit(&self) -> error::Result<CommitHash> {
        let null_staging = TreeObj::default();
        let null_staging_meta = null_staging.as_meta()?;
        let null_commit = self.create_null_commit(null_staging_meta);
        self.head.write(CommitHash(null_commit.as_meta()?.hash))?;
        let commit_hash = self.commit(null_commit)?;
        self.update_trace(null_staging, &commit_hash)?;
        Ok(commit_hash)
    }

    pub(crate) fn create_null_commit(&self, null_staging: ObjMeta) -> CommitObj {
        CommitObj {
            parents: Vec::with_capacity(0),
            text: CommitText::from("INIT"),
            committed_objs_tree: null_staging.hash,
        }
    }


    fn update_trace(&self, staging_tree: TreeObj, commit_hash: &CommitHash) -> error::Result {
        let mut trace_tree = self.trace_tree.read(commit_hash).unwrap_or_default();
        trace_tree.replace_by(staging_tree);
        self.trace_tree.write(&trace_tree, commit_hash)?;
        Ok(())
    }


    fn commit(&self, commit: CommitObj) -> error::Result<CommitHash> {
        let commit_obj = commit.as_meta()?;
        self.object.write(&commit_obj)?;
        self.head.write(CommitHash(commit_obj.hash.clone()))?;
        self.local_commits
            .append(CommitHash(commit_obj.hash.clone()))?;
        Ok(CommitHash(commit_obj.hash.clone()))
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::atomic::head::{CommitText, HeadIo};
    use crate::io::atomic::local_commits::LocalCommitsIo;
    use crate::io::atomic::object::ObjIo;
    use crate::io::atomic::staging::StagingIo;
    use crate::object::commit::CommitObj;
    use crate::object::local_commits::LocalCommitsObj;
    use crate::object::tree::TreeObj;
    use crate::object::{AsMeta, ObjHash};
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn failed_if_never_staged() {
        let mock = MockFileSystem::default();
        let commit = Commit::new(BranchName::main(), mock);
        match commit.execute("") {
            Err(error::Error::NotfoundStages) => {}
            _ => panic!("expect return error::Error::NotfoundStages bad none"),
        }
    }


    #[test]
    fn reset_staging_after_committed() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());
        let staging = StagingIo::new(mock.clone());
        mock.write("./hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("test").unwrap();
        let staging_tree = staging.read().unwrap().unwrap();

        assert_eq!(staging_tree.len(), 0);
    }


    #[test]
    fn update_head_commit_hash() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());
        mock.write("./hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("test").unwrap();

        let head = HeadIo::new(BranchName::main(), mock.clone());
        let head_hash = head.read().unwrap();
        let commit = ObjIo::new(mock).read_to_commit(&head_hash).unwrap();

        let mut tree = TreeObj::default();
        tree.insert(FilePath::from_path("./hello"), ObjHash::new(b"hello"));
        assert_eq!(
            commit,
            CommitObj {
                parents: vec![null_commit_hash],
                text: CommitText::from("test"),
                committed_objs_tree: tree.as_meta().unwrap().hash,
            }
        );
    }

    #[test]
    fn append_to_local_commit() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());
        let local_commits = LocalCommitsIo::new(BranchName::main(), mock.clone());
        mock.write("./hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash = commit.execute("test").unwrap();

        let local = local_commits.read().unwrap().unwrap();
        assert_eq!(local, LocalCommitsObj(vec![null_commit_hash, commit_hash]))
    }


    #[test]
    fn exists_2_local_commits() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());
        let local_commits = LocalCommitsIo::new(BranchName::main(), mock.clone());
        mock.write("./hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash1 = commit.execute("1").unwrap();

        mock.write("./hello2", b"hello2").unwrap();
        println!("{mock:?}");
        stage.execute(".").unwrap();
        let commit_hash2 = commit.execute("2").unwrap();

        let local = local_commits.read().unwrap().unwrap();
        assert_eq!(
            local,
            LocalCommitsObj(vec![null_commit_hash, commit_hash1, commit_hash2])
        )
    }
}
