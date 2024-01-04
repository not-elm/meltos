use crate::commands::{load_branch_name, CommandExecutable};
use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::file::StdFileSystem;

#[derive(Args, Debug, Clone)]
pub struct StageArgs {
    path: String,
}

#[async_trait]
impl CommandExecutable for StageArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(load_branch_name()?, StdFileSystem);
        tvc.stage(self.path)?;
        println!("staged");
        Ok(())
    }
}