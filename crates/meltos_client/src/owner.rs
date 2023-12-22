use async_trait::async_trait;

use meltos::schema::response::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::object::commit::CommitHash;
use meltos_tvn::operation::merge::MergedStatus;
use meltos_tvn::operation::Operations;
use meltos_tvn::operation::push::Pushable;

use crate::config::{SessionConfigs, SessionConfigsIo};
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
            configs,
            client: HttpClient::new("http://localhost:3000"),
            operations: Operations::new(BranchName::main(), fs),
        }
    }


    pub async fn open<Config: SessionConfigsIo>(
        fs: Fs,
        session: Config,
        user_id: Option<UserId>,
    ) -> crate::error::Result<Self> {
        let operations = Operations::new_main(fs);
        operations.init.execute()?;

        let client = HttpClient::new("http://localhost:3000");
        let opened = operations
            .push
            .execute(&mut OpenClient {
                http: &client,
                user_id,
            })
            .await?;
        let configs = SessionConfigs::from(opened);
        session.save(configs.clone()).await?;

        Ok(Self {
            configs,
            client,
            operations,
        })
    }



    pub fn stage(&self, workspace: &str) -> meltos_tvn::error::Result {
        self.operations.stage.execute(workspace)
    }


    pub fn commit(&self, commit_text: impl Into<CommitText>) -> meltos_tvn::error::Result<CommitHash> {
        self.operations.commit.execute(commit_text)
    }

    pub async fn push(&self) -> meltos_tvn::error::Result {
        self.operations.push.execute(&mut PushClient{
            http: &self.client,
            session_id: self.configs.session_id.clone()
        })
            .await
    }

    pub fn merge(&self, branch_name: BranchName) -> meltos_tvn::error::Result<MergedStatus> {
        self.operations.merge.execute(branch_name, BranchName::from(self.configs.user_id.to_string()))
    }
}


struct OpenClient<'a> {
    user_id: Option<UserId>,
    http: &'a HttpClient,
}


#[async_trait]
impl<'a> Pushable<Opened> for OpenClient<'a> {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<Opened, Self::Error> {
        self.http.open_room(self.user_id.clone(), bundle).await
    }
}

struct PushClient<'a> {
    http: &'a HttpClient,
    session_id: SessionId,
}

#[async_trait]
impl<'a> Pushable<()> for PushClient<'a> {
    type Error = crate::error::Error;

    async fn push(&mut self, bundle: Bundle) -> Result<(), Self::Error> {
        self.http.push(self.session_id.clone(), &bundle).await
    }
}