use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, load_configs};

#[derive(Debug, Clone, Args)]
pub struct LeaveArgs {}


#[async_trait(? Send)]
impl CommandExecutable for LeaveArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let tvc = TvcClient::new(configs.user_id.to_string(), StdFileSystem);
        tvc.leave(configs).await?;
        Ok(())
    }
}