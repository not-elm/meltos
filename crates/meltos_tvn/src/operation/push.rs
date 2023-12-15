use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::commit_obj::CommitObjIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::{CompressedBuf, ObjHash};
use crate::object::tree::TreeObj;

pub struct Push<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
    commit_obj: CommitObjIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
}


impl<Fs, Io> Push<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Push<Fs, Io> {
        Self {
            commit_obj: CommitObjIo::new(branch_name.clone(), fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name.clone(), fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(branch_name, fs),
        }
    }
}


impl<Fs, Io> Push<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    ///
    /// * clear local commits
    /// *
    pub fn execute(&self) -> error::Result {
        let local_commits = self.commit_obj.read_local_commits()?;
        if local_commits.is_empty() {
            return Err(error::Error::NotfoundLocalCommits);
        }
        let _push_param = self.create_push_param();
        self.commit_obj.reset_local_commits()?;
        Ok(())
    }


    fn create_push_param(&self) -> error::Result<PushParam> {
        let Some(head) = self.head.read()? else {
            return Err(error::Error::NotfoundHead);
        };
        let compressed_objs = self.read_objs_associated_commits(head.clone())?;
        let trace = self.trace_tree.read()?;
        Ok(PushParam {
            compressed_objs,
            head,
            trace
        })
    }


    fn read_objs_associated_commits(&self, head: ObjHash) -> error::Result<Vec<CompressedBuf>> {
        let obj_hashes = self.commit_obj.read_obj_hashes_associate_with(head)?;
        let mut obj_bufs = Vec::with_capacity(obj_hashes.len());
        for hash in obj_hashes {
            let Some(buf) = self.object.read(&hash)? else {
                return Err(error::Error::NotfoundObj(hash));
            };
            obj_bufs.push(buf);
        }
        Ok(obj_bufs)
    }
}


struct PushParam {
    pub compressed_objs: Vec<CompressedBuf>,
    pub trace: Option<TreeObj>,
    pub head: ObjHash,
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::commit_obj::CommitObjIo;
    use crate::operation::commit::Commit;
    use crate::operation::push::Push;
    use crate::operation::stage::Stage;

    #[test]
    fn failed_if_no_commit() {
        let mock = MockFileSystem::default();
        let push = Push::new(BranchName::main(), mock);
        match push.execute() {
            Err(error::Error::NotfoundLocalCommits) => {}
            _ => panic!("expected return error::Error::NotfoundLocalCommits bad was")
        }
    }

    #[test]
    fn success_if_committed() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock);

        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        assert!(push.execute().is_ok());
    }


    #[test]
    fn local_commits_is_cleared_if_succeed() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let commit_obj = CommitObjIo::new(branch.clone(), mock.clone());
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock);

        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        push.execute().unwrap();

        assert_eq!(commit_obj.read_local_commits().unwrap().len(), 0);
    }
}