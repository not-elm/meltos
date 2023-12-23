use async_trait::async_trait;

use meltos::user::{SessionId, UserId};
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::object::commit::CommitHash;
use meltos_tvn::object::local_commits::LocalCommitsObj;
use meltos_tvn::operation::merge::MergedStatus;
use meltos_tvn::operation::push::Pushable;
use meltos_tvn::operation::Operations;

use crate::config::{SessionConfigs, SessionConfigsIo};
use crate::error;
use crate::http::HttpClient;

pub struct RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    configs: SessionConfigs,
    client: HttpClient,
    operations: Operations<Fs, Io>,
}


impl<Fs, Io> RoomOwner<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Read + std::io::Write,
{
    pub fn new(fs: Fs, configs: SessionConfigs) -> RoomOwner<Fs, Io> {
        Self {
            client: HttpClient::new("http://localhost:3000", configs.session_id.clone()),
            operations: Operations::new(BranchName::main(), fs),
            configs,
        }
    }


    pub async fn open<Config: SessionConfigsIo>(
        fs: Fs,
        session: Config,
        user_id: Option<UserId>,
    ) -> crate::error::Result<Self> {
        let operations = Operations::new_main(fs);
        operations.init.execute()?;
        let bundle = operations.bundle.create()?;
        operations
            .local_commits
            .write(&LocalCommitsObj::default())?;
        let (client, configs) = HttpClient::open("http://localhost:3000", bundle, user_id).await?;
        session.save(configs.clone()).await?;

        Ok(Self {
            configs,
            client,
            operations,
        })
    }

    pub async fn fetch(&self) -> error::Result {
        let bundle = self.client.fetch().await?;
        self.operations.patch.execute(&bundle)?;
        Ok(())
    }

    pub fn stage(&self, workspace: &str) -> meltos_tvn::error::Result {
        self.operations.stage.execute(workspace)
    }

    pub fn commit(
        &self,
        commit_text: impl Into<CommitText>,
    ) -> meltos_tvn::error::Result<CommitHash> {
        self.operations.commit.execute(commit_text)
    }

    pub async fn push(&mut self) -> meltos_tvn::error::Result {
        self.operations
            .push
            .execute(&mut self.client)
            .await
    }

    pub fn merge(&self, branch_name: BranchName) -> meltos_tvn::error::Result<MergedStatus> {
        self.operations.merge.execute(
            branch_name,
            BranchName::from(self.configs.user_id.to_string()),
        )
    }


    #[inline]
    pub const fn configs(&self) -> &SessionConfigs{
        &self.configs
    }
}

