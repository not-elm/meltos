use async_trait::async_trait;
use clap::Args;

use meltos_core::user::UserId;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{save_configs, CommandExecutable};

#[derive(Debug, Clone, Args)]
pub struct JoinArgs {
    room_id: String,

    #[clap(short, long)]
    user_id: Option<String>,
}

#[async_trait(? Send)]
impl CommandExecutable for JoinArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let mut tvc = TvcClient::new(StdFileSystem);
        let configs = tvc
            .join_room(self.room_id, self.user_id.map(UserId))
            .await?;
        save_configs(&configs)?;
        println!("joined = {configs:?}");
        Ok(())
    }
}
