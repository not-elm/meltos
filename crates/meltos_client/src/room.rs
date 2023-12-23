use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::object::commit::CommitHash;
use meltos_tvn::object::local_commits::LocalCommitsObj;
use meltos_tvn::operation::merge::MergedStatus;
use meltos_tvn::operation::Operations;

use crate::config::{SessionConfigs, SessionConfigsIo};
use crate::http::HttpClient;

pub mod discussion;

pub struct RoomClient<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    client: HttpClient,
    operations: Operations<Fs, Io>,
}

impl<Fs, Io> RoomClient<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    const BASE: &'static str = "http://127.0.0.1:3000";
    pub fn new(fs: Fs, configs: SessionConfigs) -> RoomClient<Fs, Io> {
        Self {
            operations: Operations::new(BranchName::from(configs.user_id.to_string()), fs),
            client: HttpClient::new(Self::BASE, configs),
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
        let client = HttpClient::open(Self::BASE, bundle, user_id).await?;
        session.save(client.configs().clone()).await?;

        Ok(Self {
            client,
            operations,
        })
    }

    pub async fn join<Config: SessionConfigsIo>(
        session: Config,
        fs: Fs,
        room_id: RoomId,
        user_id: Option<UserId>,
    ) -> Result<Self, crate::error::Error> {
        let (client, bundle) = HttpClient::join(Self::BASE, room_id.clone(), user_id).await?;
        let configs = client.configs();
        session.save(configs.clone()).await?;

        let branch_name = BranchName::from(configs.user_id.to_string());
        let operations = Operations::new(branch_name.clone(), fs);
        operations.save.execute(bundle)?;
        operations.checkout.execute(&branch_name)?;
        operations.unzip.execute(&branch_name)?;

        Ok(Self {
            client,
            operations,
        })
    }

    #[inline]
    pub async fn fetch(&self) -> crate::error::Result {
        let bundle = self.client.fetch().await?;
        self.operations.patch.execute(&bundle)?;
        Ok(())
    }

    #[inline(always)]
    pub fn merge(&self, source: BranchName) -> meltos_tvn::error::Result<MergedStatus> {
        self.operations
            .merge
            .execute(source, BranchName::from(self.configs().user_id.to_string()))
    }

    #[inline(always)]
    pub fn stage(&self, workspace_path: &str) -> meltos_tvn::error::Result {
        self.operations.stage.execute(workspace_path)
    }

    #[inline(always)]
    pub fn commit(
        &self,
        commit_text: impl Into<CommitText>,
    ) -> meltos_tvn::error::Result<CommitHash> {
        self.operations.commit.execute(commit_text)
    }

    pub async fn push(&mut self) -> meltos_tvn::error::Result {
        self.operations.push.execute(&mut self.client).await
    }

    #[inline(always)]
    pub const fn configs(&self) -> &SessionConfigs {
        self.client.configs()
    }
}
