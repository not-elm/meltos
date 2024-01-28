use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::io::atomic::head::HeadIo;

use crate::commands::{CommandExecutable, load_branch_name};

#[derive(Debug, Args, Clone)]
pub struct MergeArgs {
    source_branch: String,
}

#[async_trait(? Send)]
impl CommandExecutable for MergeArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let tvc = TvcClient::new(StdFileSystem);
        let status =
            tvc
                .merge(load_branch_name()?,HeadIo::new(StdFileSystem).try_read(&BranchName(self.source_branch)).await?)
                .await?;
        println!("merged status = {status:?}");
        Ok(())
    }
}
