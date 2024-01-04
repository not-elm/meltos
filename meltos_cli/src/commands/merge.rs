use crate::commands::{load_branch_name, CommandExecutable};
use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

#[derive(Debug, Args, Clone)]
pub struct MergeArgs {
    source_branch: String,
}

#[async_trait]
impl CommandExecutable for MergeArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let mut tvc = TvcClient::new(load_branch_name()?, StdFileSystem);
        let status = tvc.merge(self.source_branch)?;
        println!("merged status = {status:?}");
        Ok(())
    }
}
