use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, save_configs};

#[derive(Args, Debug, Clone)]
pub struct OpenArgs {
    #[clap(short, long)]
    lifetime_secs: Option<u64>,

    #[clap(short, long)]
    user_limits: Option<u64>,
}

#[async_trait(? Send)]
impl CommandExecutable for OpenArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let mut tvc = TvcClient::new(StdFileSystem);
        let session_configs = tvc.open_room(self.lifetime_secs, self.user_limits).await?;
        save_configs(&session_configs)?;
        println!("opened = {session_configs:?}");
        Ok(())
    }
}
