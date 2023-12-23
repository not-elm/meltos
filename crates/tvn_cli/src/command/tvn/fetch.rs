use async_trait::async_trait;
use clap::Args;

use meltos_client::config::tmp_file::TmpSessionConfigsIo;
use meltos_client::config::SessionConfigsIo;
use meltos_client::room::RoomClient;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;

#[derive(Debug, Clone, Args)]
pub struct FetchArgs;

#[async_trait]
impl CommandExecutable for FetchArgs {
    async fn execute(self) -> crate::error::Result {
        let room = RoomClient::new(StdFileSystem, TmpSessionConfigsIo.load().await?);
        room.fetch().await?;
        Ok(())
    }
}
