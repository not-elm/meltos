use std::fmt::Display;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::local_commits::LocalCommitsIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::bundle::{Bundle, BundleBranch, BundleObject, BundleTrace};
use crate::io::commit_obj::CommitObjIo;
use crate::object::commit::CommitObj;

#[async_trait]
pub trait Pushable<Output> {
    type Error: Display;

    async fn push(&mut self, bundle: Bundle) -> std::result::Result<Output, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Push<Fs>
where
    Fs: FileSystem,
{
    commit_obj: CommitObjIo<Fs>,
    local_commits: LocalCommitsIo<Fs>,
    branch_name: BranchName,
    trace: TraceIo<Fs>,
}

impl<Fs> Push<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Push<Fs> {
        Self {
            commit_obj: CommitObjIo::new(branch_name.clone(), fs.clone()),
            trace: TraceIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name.clone(), fs),
            branch_name,
        }
    }
}

impl<Fs> Push<Fs>
where
    Fs: FileSystem,
{
    /// Sends the currently locally committed data to the remote.
    /// * push local commits to remote server.
    /// * clear local commits
    pub async fn execute<Output>(
        &self,
        remote: &mut impl Pushable<Output>,
    ) -> error::Result<Output> {
        let bundle = self.create_push_bundle()?;
        let output = remote
            .push(bundle)
            .await
            .map_err(|e| error::Error::FailedConnectServer(format!("{e}")))?;
        self.commit_obj.reset_local_commits()?;
        Ok(output)
    }

    pub fn create_push_bundle(&self) -> error::Result<Bundle> {
        let local_commits = self.local_commits.read()?.unwrap_or_default();
        if local_commits.is_empty() {
            return Err(error::Error::NotfoundLocalCommits);
        }
        let traces = self.trace.read_all()?;
        let objs = self.commit_obj.read_objs_associated_with_local_commits()?;
        Ok(Bundle {
            objs,
            traces,
            branches: vec![BundleBranch {
                branch_name: self.branch_name.clone(),
                commits: local_commits.0,
            }],
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PushBundle {
    pub traces: Vec<BundleTrace>,
    pub objs: Vec<BundleObject>,
    pub branch_name: BranchName,
    pub commits: Vec<CommitObj>,
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::bundle::Bundle;
    use crate::io::commit_obj::CommitObjIo;
    use crate::operation::commit::Commit;
    use crate::operation::push::{Push, Pushable};
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[derive(Debug, Default)]
    struct MockRemoteClient {
        pub bundle: Option<Bundle>,
    }

    unsafe impl Send for MockRemoteClient {}

    unsafe impl Sync for MockRemoteClient {}

    #[async_trait]
    impl Pushable<()> for MockRemoteClient {
        type Error = String;

        async fn push(&mut self, bundle: Bundle) -> Result<(), Self::Error> {
            self.bundle.replace(bundle);
            Ok(())
        }
    }

    #[tokio::test]
    async fn failed_if_no_commit() {
        let mock = MockFileSystem::default();
        let push = Push::new(BranchName::owner(), mock);
        match push.execute(&mut MockRemoteClient::default()).await {
            Err(error::Error::NotfoundLocalCommits) => {}
            _ => panic!("expected return error::Error::NotfoundLocalCommits bad was"),
        }
    }

    #[tokio::test]
    async fn success_if_committed() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::owner();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock.clone());

        mock.write_file("workspace/hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        assert!(push.execute(&mut MockRemoteClient::default()).await.is_ok());
    }

    #[tokio::test]
    async fn local_commits_is_cleared_if_succeed() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::owner();
        let commit_obj = CommitObjIo::new(branch.clone(), mock.clone());
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch, mock.clone());
        mock.write_file("workspace/hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        push.execute(&mut MockRemoteClient::default())
            .await
            .unwrap();

        assert_eq!(commit_obj.read_local_commits().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn push_param() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::owner();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let push = Push::new(branch.clone(), mock.clone());

        mock.write_file("workspace/hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();
        let mut remote = MockRemoteClient::default();
        push.execute(&mut remote).await.unwrap();
        let bundle = remote.bundle.unwrap();
        let head = HeadIo::new(mock);
        let commits = &bundle.branches[0].commits;
        assert_eq!(
            &commits[commits.len() - 1],
            &head.try_read(&branch).unwrap()
        );
    }
}
