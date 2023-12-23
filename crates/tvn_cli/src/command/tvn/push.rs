use async_trait::async_trait;
use clap::Args;

use meltos_client::config::tmp_file::TmpSessionConfigsIo;
use meltos_client::config::SessionConfigsIo;
use meltos_client::room::RoomClient;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;

#[derive(Args, Clone, Debug)]
pub struct PushArgs;

#[async_trait]
impl CommandExecutable for PushArgs {
    async fn execute(self) -> crate::error::Result {
        let mut room = RoomClient::new(StdFileSystem, TmpSessionConfigsIo.load().await?);
        room.push().await?;
        println!("success");
        Ok(())
    }
}
