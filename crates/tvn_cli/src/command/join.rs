use async_trait::async_trait;
use clap::Args;

use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_client::config::tmp_file::TmpSessionConfigsIo;
use meltos_client::room::RoomClient;
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
        let user = RoomClient::join(
            TmpSessionConfigsIo,
            StdFileSystem,
            RoomId(self.room_id),
            self.user_id,
        )
        .await?;
        println!("room = {:?}", user.configs());
        Ok(())
    }
}
