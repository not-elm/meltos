use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, load_configs};

#[derive(Args, Debug, Clone)]
pub struct PushArgs;

#[async_trait(? Send)]
impl CommandExecutable for PushArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let configs = load_configs()?;
        let mut tvc = TvcClient::new(StdFileSystem);
        tvc.push(configs).await?;
        Ok(())
    }
}
