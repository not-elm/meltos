use reqwest::{header, Client};

use meltos::branch::structs::branch_name::BranchName;
use meltos::room::RoomId;
use meltos::user::SessionId;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::io::atomic::head::CommitText;
use meltos_tvn::io::bundle::Bundle;
use meltos_tvn::operation::Operations;

use crate::error::Error;

pub struct TvnClient {
    client: Client,
    room_id: RoomId,
    session_id: SessionId,
    operations: Operations,
}


impl TvnClient {
    pub fn new(room_id: RoomId, session_id: SessionId) -> Self {
        Self {
            room_id,
            session_id,
            client: Client::new(),
            operations: Operations::new_main(StdFileSystem),
        }
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
