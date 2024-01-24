use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{load_configs, CommandExecutable};

#[derive(Debug, Clone, Args)]
pub struct LeaveArgs {}

#[async_trait(? Send)]
impl CommandExecutable for LeaveArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let tvc = TvcClient::new(StdFileSystem, Some(BranchName(configs.user_id.to_string())));
        tvc.leave(configs).await?;
        Ok(())
    }
}
