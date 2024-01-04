use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvnClient;
use meltos_tvn::file_system::file::StdFileSystem;
use crate::commands::{CommandExecutable, load_branch_name};

#[derive(Debug, Clone, Args)]
pub struct CommitArgs{
    commit_text: String
}


#[async_trait]
impl CommandExecutable for CommitArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvnClient::new(load_branch_name()?, StdFileSystem);
        tvc.commit(self.commit_text)?;
        println!("committed");
        Ok(())
    }
}