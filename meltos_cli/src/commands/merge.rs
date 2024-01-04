use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;
use crate::commands::{CommandExecutable, load_branch_name};

#[derive(Debug, Args, Clone)]
pub struct MergeArgs{
    source_branch: String
}



#[async_trait]
impl CommandExecutable for MergeArgs{
    async fn execute(self) -> meltos_client::error::Result {
        let mut tvc = TvnClient::new(load_branch_name()?, StdFileSystem);
        let status = tvc.merge(self.source_branch)?;
        println!("merged status = {status:?}");
        Ok(())
    }
}