use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, load_configs};

#[derive(Args, Debug, Clone)]
pub struct FetchArgs;

#[async_trait(? Send)]
impl CommandExecutable for FetchArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let tvc = TvcClient::new(StdFileSystem);
        tvc.fetch(configs).await?;
        println!("fetched");
        Ok(())
    }
}
