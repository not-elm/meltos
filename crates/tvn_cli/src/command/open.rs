use std::env;

use async_trait::async_trait;
use clap::Args;
use log::info;
use meltos::user::UserId;
use meltos_client::config::tmp_file::TmpSessionConfigsIo;

use meltos_client::owner::RoomOwner;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;
use crate::error;

#[derive(Debug, Args, Eq, PartialEq, Clone)]
pub struct OpenArgs {
    user_id: Option<UserId>,
}


#[async_trait]
impl CommandExecutable for OpenArgs {
    async fn execute(self) -> error::Result {
        let owner = RoomOwner::open(StdFileSystem, TmpSessionConfigsIo, self.user_id).await?;

        info!("open={:?}", owner.configs());

        Ok(())
    }
}
