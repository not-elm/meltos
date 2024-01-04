use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;
use crate::commands::{CommandExecutable, load_configs};

#[derive(Args, Debug, Clone)]
pub struct PushArgs;

#[async_trait]
impl CommandExecutable for PushArgs{
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let mut tvc = TvnClient::new(configs.user_id.to_string(), StdFileSystem);
        tvc.push(configs).await?;
        Ok(())
    }
}