use std::env;

use async_trait::async_trait;
use clap::Args;
use log::info;
use meltos::user::UserId;

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
        let owner = RoomOwner::open(StdFileSystem, self.user_id).await?;

        info!("open user : room_id={}", owner.room_id);
        env::set_var("ROOM_ID", owner.room_id.0);
        env::set_var("SESSION_ID", owner.session_id.0);
        env::set_var("USER_ID", owner.user_id.to_string());

        Ok(())
    }
}
