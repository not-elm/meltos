use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::file::StdFileSystem;

use crate::commands::{save_configs, CommandExecutable};

#[derive(Args, Debug, Clone)]
pub struct OpenArgs {
    #[clap(short, long)]
    lifetime_secs: Option<u64>,
}

#[async_trait]
impl CommandExecutable for OpenArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(BranchName::owner().0, StdFileSystem);
        let session_configs = tvc.open_room(self.lifetime_secs).await?;
        save_configs(&session_configs)?;
        println!("opened = {session_configs:?}");
        Ok(())
    }
}
