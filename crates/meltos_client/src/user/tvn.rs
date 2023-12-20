use reqwest::{header, Client};


use meltos::room::RoomId;
use meltos::user::SessionId;
use meltos_tvn::branch::BranchName;

use meltos_tvn::file_system::FileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::Operations;

use crate::error::Error;

pub struct TvnClient<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    client: Client,
    room_id: RoomId,
    session_id: SessionId,
    operations: Operations<Fs, Io>,
}


impl<Fs, Io> TvnClient<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(room_id: RoomId, session_id: SessionId, fs: Fs) -> Self {
        Self {
            room_id,
            session_id,
            client: Client::new(),
            operations: Operations::new_main(fs),
        }
    }


    pub fn init(&self, branch_name: &BranchName, bundle: Bundle) -> crate::error::Result {
        self.operations.save.execute(bundle)?;
        self.operations.checkout.execute(branch_name)?;
        self.operations.unzip.execute(branch_name)?;
        Ok(())
    }


    async fn fetch(&self, target_branch: Option<BranchName>) -> Result<(), Error> {
        let response = self
            .client
            .get(format!("http://localhost:3000/room/{}/fetch", self.room_id))
            .header(
                header::SET_COOKIE,
                format!("session_id={}", self.session_id),
            )
            .send()
            .await?;
        let bundle = response.json::<Bundle>().await?;
        self.operations.patch.execute(&bundle)?;
        Ok(())
    }

    async fn stage(&self, workspace_path: &str) -> Result<(), Error> {
        self.operations.stage.execute(workspace_path)?;
        Ok(())
    }

    async fn commit(&self, commit_text: impl Into<CommitText>) -> Result<(), Error> {
        self.operations.commit.execute(commit_text)?;
        Ok(())
    }

    async fn push(&self) -> Result<(), Error> {
        todo!()
    }
}
