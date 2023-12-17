use serde::{Deserialize, Serialize};

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::commit_obj::CommitObjIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::commit::CommitHash;
use crate::object::{CompressedBuf, ObjHash};

use crate::remote_client::CommitSendable;

#[derive(Debug, Clone)]
pub struct Push<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
    commit_obj: CommitObjIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    branch_name: BranchName,
    trace: TraceIo<Fs, Io>,
}


impl<Fs, Io> Push<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Push<Fs, Io> {
        Self {
            commit_obj: CommitObjIo::new(branch_name.clone(), fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(branch_name.clone(), fs.clone()),
            trace: TraceIo::new(fs),
            branch_name,
        }
    }
}


impl<Fs, Io> Push<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    /// Sends the currently locally committed data to the remote.
    /// * push local commits to remote server.
    /// * clear local commits
    pub async fn execute(&self, sender: &impl CommitSendable) -> error::Result {
        let local_commits = self.commit_obj.read_local_commits()?;
        if local_commits.is_empty() {
            return Err(error::Error::NotfoundLocalCommits);
        }
        let push_param = self.create_push_param()?;
        sender.send(push_param).await?;
        self.commit_obj.reset_local_commits()?;
        Ok(())
    }


    pub fn create_push_param(&self) -> error::Result<PushParam> {
        let traces = self.trace.read_all()?;
        let head = self.head.read()?;
        let compressed_objs = self.read_objs_associated_commits(head.clone())?;
        Ok(PushParam {
            branch: self.branch_name.clone(),
            compressed_objs,
            head,
            traces,
        })
    }


    fn read_objs_associated_commits(&self, head: CommitHash) -> error::Result<Vec<CompressedBuf>> {
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushParam {
    pub branch: BranchName,
    pub compressed_objs: Vec<CompressedBuf>,
    pub traces: Vec<(CommitHash, ObjHash)>,
    pub head: CommitHash,
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::commit_obj::CommitObjIo;
    use crate::io::trace_tree::TraceTreeIo;
    use crate::operation::commit::Commit;
    use crate::operation::push::Push;
    use crate::operation::stage::Stage;
    use crate::remote_client::mock::MockRemoteClient;
    use crate::tests::init_main_branch;

    #[tokio::test]
    async fn failed_if_no_commit() {
        let mock = MockFileSystem::default();
        let push = Push::new(BranchName::main(), mock);
        match push.execute(&MockRemoteClient::default()).await {
            Err(error::Error::NotfoundLocalCommits) => {}
            _ => panic!("expected return error::Error::NotfoundLocalCommits bad was"),
        }
    }

    #[tokio::test]
    async fn success_if_committed() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        assert!(push.execute(&MockRemoteClient::default()).await.is_ok());
    }


    #[tokio::test]
    async fn local_commits_is_cleared_if_succeed() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::main();
        let commit_obj = CommitObjIo::new(branch.clone(), mock.clone());
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock.clone());
        mock.write("./.hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        push.execute(&MockRemoteClient::default()).await.unwrap();

        assert_eq!(commit_obj.read_local_commits().unwrap().len(), 0);
    }


    #[tokio::test]
    async fn push_param() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch.clone(), mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        let remote = MockRemoteClient::default();
        push.execute(&remote).await.unwrap();
        let param = remote.push_param.lock().await.clone().unwrap();
        let trace_tree = TraceTreeIo::new(mock.clone());
        let head = HeadIo::new(branch, mock);
        assert_eq!(&param.head, &head.read().unwrap());
        let head = head.read().unwrap();
    }
}
