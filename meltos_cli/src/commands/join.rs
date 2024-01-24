use async_trait::async_trait;
use clap::Args;

use meltos::user::UserId;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, save_configs};

#[derive(Debug, Clone, Args)]
pub struct JoinArgs {
    room_id: String,

    #[clap(short, long)]
    user_id: Option<String>,
}

#[async_trait(? Send)]
impl CommandExecutable for JoinArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let mut tvc = TvcClient::new(StdFileSystem, None);
        let configs = tvc.join_room(self.room_id, self.user_id.map(UserId)).await?;
        save_configs(&configs)?;
        println!("joined = {configs:?}");
        Ok(())
    }
}
