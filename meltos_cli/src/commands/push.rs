use crate::commands::{load_configs, CommandExecutable};
use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

#[derive(Args, Debug, Clone)]
pub struct PushArgs;

#[async_trait]
impl CommandExecutable for PushArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let mut tvc = TvcClient::new(configs.user_id.to_string(), StdFileSystem);
        tvc.push(configs).await?;
        Ok(())
    }
}
