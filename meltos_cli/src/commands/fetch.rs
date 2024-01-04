use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::file::StdFileSystem;

use crate::commands::{load_configs, CommandExecutable};

#[derive(Args, Debug, Clone)]
pub struct FetchArgs;

#[async_trait]
impl CommandExecutable for FetchArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let tvc = TvcClient::new(configs.user_id.clone().0, StdFileSystem);
        tvc.fetch(configs).await?;
        println!("fetched");
        Ok(())
    }
}
