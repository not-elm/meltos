use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvnClient;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::commands::{CommandExecutable, save_configs};

#[derive(Args, Debug, Clone)]
pub struct OpenArgs {
    #[clap(short, long)]
    lifetime_secs: Option<u64>,
}


#[async_trait]
impl CommandExecutable for OpenArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvn = TvnClient::new(BranchName::owner().0, StdFileSystem);
        let session_configs = tvn.open_room(self.lifetime_secs).await?;
        save_configs(&session_configs)?;
        println!("opened = {session_configs:?}");
        Ok(())
    }
}