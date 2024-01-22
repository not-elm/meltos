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
pub struct Commit<Fs>
where
    Fs: FileSystem,
{
    commit_obj: CommitObjIo<Fs>,
    head: HeadIo<Fs>,
    object: ObjIo<Fs>,
    staging: StagingIo<Fs>,
    trace_tree: TraceTreeIo<Fs>,
    local_commits: LocalCommitsIo<Fs>,
}

impl<Fs> Commit<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Commit<Fs> {
        Self {
            commit_obj: CommitObjIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            staging: StagingIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(fs),
        }
    }
}

impl<Fs> Commit<Fs>
where
    Fs: FileSystem,
{
    pub fn execute(
        &self,
        branch_name: &BranchName,
        commit_text: impl Into<CommitText>,
    ) -> error::Result<CommitHash> {
        let Some(stage_tree) = self.staging.read()? else {
            return Err(error::Error::NotfoundStages);
        };
        self.staging.reset()?;
        let stage_meta = stage_tree.as_meta()?;
        self.object.write_obj(&stage_tree)?;

        let commit = self
            .commit_obj
            .create(commit_text, stage_meta.hash, branch_name)?;
        let pre_head = self.head.read(branch_name)?;
        let head_commit_hash = self.commit(branch_name, commit)?;
        self.update_trace(stage_tree, &head_commit_hash, &pre_head)?;
        Ok(head_commit_hash)
    }

    /// * create `null commit`
    /// * create `head file` and write `null commit hash`
    /// * create `trace file` named `null commit hash`.
    /// * create `local commits file` and append `null commit hash`
    pub fn execute_null_commit(&self, branch_name: &BranchName) -> error::Result<CommitHash> {
        let null_staging = TreeObj::default();
        let null_staging_meta = null_staging.as_meta()?;
        let null_commit = self.create_null_commit(null_staging_meta);
        self.head
            .write(branch_name, &CommitHash(null_commit.as_meta()?.hash))?;
        let commit_hash = self.commit(branch_name, null_commit)?;
        self.update_trace(null_staging, &commit_hash, &None)?;
        self.staging.reset()?;
        Ok(commit_hash)
    }

    pub(crate) fn create_null_commit(&self, null_staging: ObjMeta) -> CommitObj {
        CommitObj {
            parents: Vec::with_capacity(0),
            text: CommitText::from("Initial Commit"),
            committed_objs_tree: null_staging.hash,
        }
    }

    fn update_trace(
        &self,
        staging_tree: TreeObj,
        commit_hash: &CommitHash,
        pre_head: &Option<CommitHash>,
    ) -> error::Result {
        let mut trace_tree = match pre_head {
            Some(head) => self.trace_tree.read(head).unwrap_or_default(),
            None => TreeObj::default(),
        };

        trace_tree.replace_by(staging_tree);
        self.trace_tree.write(&trace_tree, commit_hash)?;

        Ok(())
    }

    fn commit(&self, branch_name: &BranchName, commit: CommitObj) -> error::Result<CommitHash> {
        let commit_meta = commit.as_meta()?;
        self.object.write_obj(&commit)?;
        self.head
            .write(branch_name, &CommitHash(commit_meta.hash.clone()))?;
        self.local_commits
            .append(CommitHash(commit_meta.hash.clone()), branch_name)?;
        Ok(CommitHash(commit_meta.hash.clone()))
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
    use crate::tests::init_owner_branch;

    #[test]
    fn failed_if_never_staged() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let commit = Commit::new(fs);
        match commit.execute(&branch, "") {
            Err(error::Error::NotfoundStages) => {}
            _ => panic!("expect return error::Error::NotfoundStages bad none"),
        }
    }

    #[test]
    fn reset_staging_after_committed() {
        let fs = MockFileSystem::default();
        init_owner_branch(fs.clone());
        let branch = BranchName::owner();
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let staging = StagingIo::new(fs.clone());
        fs.write_file("workspace/hello", b"hello").unwrap();
        stage.execute(&branch, ".").unwrap();
        commit.execute(&branch, "test").unwrap();
        let staging_tree = staging.read().unwrap().unwrap();

        assert_eq!(staging_tree.len(), 0);
    }

    #[test]
    fn update_head_commit_hash() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let null_commit_hash = init_owner_branch(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        fs.write_file("workspace/hello", b"hello").unwrap();
        stage.execute(&branch, ".").unwrap();
        commit.execute(&branch, "test").unwrap();

        let head = HeadIo::new(fs.clone());
        let head_hash = head.try_read(&BranchName::owner()).unwrap();
        let commit = ObjIo::new(fs).read_to_commit(&head_hash).unwrap();

        let mut tree = TreeObj::default();
        tree.insert(
            FilePath::from_path("workspace/hello"),
            ObjHash::new(b"FILE\0hello"),
        );
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
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let null_commit_hash = init_owner_branch(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let local_commits = LocalCommitsIo::new(fs.clone());
        fs.write_file("workspace/hello", b"hello").unwrap();
        stage.execute(&branch, ".").unwrap();
        let commit_hash = commit.execute(&branch, "test").unwrap();
        let local = local_commits.read(&branch).unwrap().unwrap();
        assert_eq!(local, LocalCommitsObj(vec![null_commit_hash, commit_hash]))
    }

    #[test]
    fn exists_2_local_commits() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let null_commit_hash = init_owner_branch(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let local_commits = LocalCommitsIo::new(fs.clone());
        fs.write_file("workspace/hello", b"hello").unwrap();
        stage.execute(&branch, ".").unwrap();
        let commit_hash1 = commit.execute(&branch, "1").unwrap();
        fs.write_file("workspace/hello2", b"hello2").unwrap();
        stage.execute(&branch, ".").unwrap();
        let commit_hash2 = commit.execute(&branch, "2").unwrap();

        let local = local_commits.read(&branch).unwrap().unwrap();
        assert_eq!(
            local,
            LocalCommitsObj(vec![null_commit_hash, commit_hash1, commit_hash2])
        )
    }
}
