use crate::commands::{load_branch_name, CommandExecutable};
use async_trait::async_trait;
use clap::Args;
use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::std_fs::StdFileSystem;

#[derive(Debug, Clone, Args)]
pub struct CommitArgs {
    commit_text: String,
}

#[async_trait(?Send)]
impl CommandExecutable for CommitArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(StdFileSystem, Some(BranchName(load_branch_name()?)));
        tvc.commit(self.commit_text).await?;
        println!("committed");
        Ok(())
    }
}
