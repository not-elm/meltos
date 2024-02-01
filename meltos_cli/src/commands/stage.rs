use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{load_branch_name, CommandExecutable};

#[derive(Args, Debug, Clone)]
pub struct StageArgs {
    path: String,
}

#[async_trait(? Send)]
impl CommandExecutable for StageArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(StdFileSystem);
        tvc.stage(&load_branch_name()?, self.path).await?;
        println!("staged");
        Ok(())
    }
}
