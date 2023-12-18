use std::env;

use async_trait::async_trait;
use clap::Args;
use log::info;

use meltos_client::owner::RoomOwner;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;
use crate::{error, read_session_id};

#[derive(Debug, Args, Eq, PartialEq, Clone)]
pub struct OpenArgs;


#[async_trait]
impl CommandExecutable for OpenArgs {
    async fn execute(self) -> error::Result {
        let owner = RoomOwner::open(StdFileSystem, &read_session_id()?).await?;
        info!("open room : room_id={}", owner.room_id);
        env::set_var("ROOM_ID", owner.room_id.0);

        Ok(())
    }
}
