use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::{CommitText, HeadIo};
use crate::io::atomic::object::ObjectIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::commit::{Commit, };
use crate::object::{AsObject, ObjectHash};


pub struct CommitIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
    object: ObjectIo<Fs, Io>,
    staging: StagingIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>
}


impl<Fs, Io> CommitIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, fs: Fs) -> CommitIo<Fs, Io>{
        CommitIo{
            head: HeadIo::new(branch_name.clone(), fs.clone()),
            staging: StagingIo::new(fs.clone()),
            object: ObjectIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name, fs)
        }
    }


    pub fn commit(&self, commit_text: impl Into<CommitText>) -> error::Result {
        let Some(stage_tree) = self.staging.read_tree()? else {
            return Err(error::Error::NotfoundStages);
        };
        self.staging.reset()?;
        let stage_obj = stage_tree.as_obj()?;
        self.trace_tree.write_trace_tree(stage_tree)?;
        self.object.write(&stage_obj)?;

        let commit = self.create_commit(commit_text, stage_obj.hash)?;
        let commit_obj = commit.as_obj()?;
        self.object.write(&commit_obj)?;
        self.head.write_head(commit_obj.hash)?;
        Ok(())
    }


    pub fn read(&self) -> error::Result<Option<Commit>> {
        let Some(hash) = self.head.head_commit_hash()?
            else {
                return Ok(None);
            };
        let commit_obj = self.object.try_read_obj(&hash)?;
        Ok(Some(Commit::try_from(commit_obj)?))
    }

    fn create_commit(
        &self,
        commit_text: impl Into<CommitText>,
        staging_hash: ObjectHash,
    ) -> error::Result<Commit> {
        let head_commit = self.head.head_commit_hash()?;
        Ok(Commit {
            parent: head_commit,
            text: commit_text.into(),
            stage: staging_hash,
        })
    }

}




#[cfg(test)]
mod tests{
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::{CommitText, HeadIo};
    use crate::io::atomic::object::ObjectIo;
    use crate::io::atomic::staging::StagingIo;

    use crate::io::commit::CommitIo;
    use crate::io::stage::StageIo;
    use crate::object::commit::Commit;
    use crate::object::ObjectHash;
    use crate::object::tree::Tree;

    #[test]
    fn failed_if_never_staged(){
        let mock = MockFileSystem::default();
        let commit = CommitIo::new(BranchName::main(), mock);

        match commit.commit("") {
            Err(error::Error::NotfoundStages) => {},
            _ => panic!("expect return error::Error::NotfoundStages bad none")
        }
    }


    #[test]
    fn reset_staging_after_committed(){
        let mock = MockFileSystem::default();
        let stage = StageIo::new(BranchName::main(), mock.clone());
        let commit = CommitIo::new(BranchName::main(), mock.clone());
        let staging = StagingIo::new(mock.clone());
        mock.write_all("./hello", b"hello").unwrap();
        stage.stage(".").unwrap();
        commit.commit("test").unwrap();
        let staging_tree = staging.read_tree().unwrap().unwrap();

        assert_eq!(staging_tree.len(), 0);
    }


    #[test]
    fn update_head_commit_hash(){
        let mock = MockFileSystem::default();
        let stage = StageIo::new(BranchName::main(), mock.clone());
        let commit = CommitIo::new(BranchName::main(), mock.clone());
        mock.write_all("./hello", b"hello").unwrap();
        stage.stage(".").unwrap();
        commit.commit("test").unwrap();

        let head = HeadIo::new(BranchName::main(), mock.clone());
        let head_hash = head.head_commit_hash().unwrap().unwrap();

        let commit = ObjectIo::new(mock).read_to_commit(&head_hash).unwrap();

        let mut tree = Tree::default();
        tree.insert(FilePath::from_path("./hello"), ObjectHash::new(b"hello"));

        assert_eq!(commit, Commit{
            parent: None,
            text: CommitText::from("test"),
            stage: tree.as_obj().unwrap().hash
        });
    }
}