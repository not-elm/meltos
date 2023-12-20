use async_trait::async_trait;
use clap::Args;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_client::user::RoomUser;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::command::CommandExecutable;

#[derive(Debug, Args, Eq, PartialEq, Clone)]
pub struct JoinArgs {
    room_id: String,

    #[arg(
        short,
        long,
        help = "ルーム内のユーザー名 省力された場合ランダムな名前になります"
    )]
    user_id: Option<UserId>,
}


#[async_trait]
impl CommandExecutable for JoinArgs {
    async fn execute(self) -> crate::error::Result {
        let user = RoomUser::join(RoomId(self.room_id), self.user_id, StdFileSystem).await?;
        std::env::set_var("ROOM_ID", user.room_id.0);
        std::env::set_var("USER_ID", user.user_id.to_string());
        std::env::set_var("SESSION_ID", user.session_id.to_string());
        Ok(())
    }
}
