use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;

use crate::commands::{CommandExecutable, load_configs};

#[derive(Args, Debug, Clone)]
pub struct FetchArgs;


#[async_trait]
impl CommandExecutable for FetchArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let tvc = TvnClient::new(configs.user_id.clone().0, StdFileSystem);
        tvc.fetch(configs).await?;
        println!("fetched");
        Ok(())
    }
}