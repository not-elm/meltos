use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::file::StdFileSystem;

use crate::commands::{save_configs, CommandExecutable};

#[derive(Debug, Clone, Args)]
pub struct JoinArgs {
    room_id: String,

    user_id: String,
}

#[async_trait]
impl CommandExecutable for JoinArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(self.user_id.clone(), StdFileSystem);
        let configs = tvc.join_room(self.room_id, self.user_id).await?;
        save_configs(&configs)?;
        println!("joined = {configs:?}");
        Ok(())
    }
}
