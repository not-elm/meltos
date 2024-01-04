use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;
use crate::commands::{CommandExecutable, load_branch_name};

#[derive(Args, Debug, Clone)]
pub struct StageArgs{
    path: String
}


#[async_trait]
impl CommandExecutable for StageArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvn = TvnClient::new(load_branch_name()?, StdFileSystem);
        tvn.stage(self.path)?;
        println!("staged");
        Ok(())
    }
}