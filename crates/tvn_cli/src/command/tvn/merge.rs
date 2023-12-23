use async_trait::async_trait;
use clap::Args;
use meltos_client::config::tmp_file::TmpSessionConfigsIo;
use meltos_client::config::SessionConfigsIo;
use meltos_client::room::RoomClient;

use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;

#[derive(Args, Debug, Clone, Eq, PartialEq)]
pub struct MergeArgs {
    source: BranchName,
}

#[async_trait]
impl CommandExecutable for MergeArgs {
    async fn execute(self) -> crate::error::Result {
        let room = RoomClient::new(StdFileSystem, TmpSessionConfigsIo.load().await?);
        let status = room.merge(self.source)?;
        println!("{status:?}");
        Ok(())
    }
}
