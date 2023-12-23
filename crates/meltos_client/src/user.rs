use reqwest::Client;

use meltos::room::RoomId;
use meltos::schema::request::room::{Join, Joined};
use meltos::user::UserId;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::object::commit::CommitHash;
use meltos_tvn::operation::Operations;

use crate::config::{SessionConfigs, SessionConfigsIo};
use crate::http::HttpClient;

pub mod discussion;
pub mod tvn;


pub struct RoomUser<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    client: HttpClient,
    operations: Operations<Fs, Io>,
    configs: SessionConfigs,
}


impl<Fs, Io> RoomUser<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub async fn join<Config: SessionConfigsIo>(
        session: Config,
        fs: Fs,
        room_id: RoomId,
        user_id: Option<UserId>,
    ) -> Result<Self, crate::error::Error> {
        let (client, joined) = HttpClient::join("http://localhost:3000", &room_id, user_id).await?;

        let branch_name = BranchName::from(joined.user_id.to_string());
        let operations = Operations::new(branch_name.clone(), fs);
        operations.save.execute(joined.bundle)?;
        operations.checkout.execute(&branch_name)?;
        operations.unzip.execute(&branch_name)?;

        let configs = SessionConfigs::new(joined.session_id, room_id, joined.user_id);
        session.save(configs.clone()).await?;

        Ok(Self {
            client,
            configs,
            operations,
        })
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
}
