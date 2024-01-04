use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::commands::{CommandExecutable, save_configs};

#[derive(Debug, Clone, Args)]
pub struct JoinArgs {
    room_id: String,

    user_id: String,
}


#[async_trait]
impl CommandExecutable for JoinArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvn = TvnClient::new(self.user_id.clone(), StdFileSystem);
        let configs = tvn.join_room(self.room_id, self.user_id).await?;
        save_configs(&configs)?;
        println!("joined = {configs:?}");
        Ok(())
    }
}