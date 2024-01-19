use async_trait::async_trait;
use clap::Args;

use meltos_client::tvc::TvcClient;
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::commands::{CommandExecutable, load_branch_name};

#[derive(Debug, Args, Clone)]
pub struct MergeArgs {
    source_branch: String,
}

#[async_trait(? Send)]
impl CommandExecutable for MergeArgs {
    async fn execute(self) -> meltos_client::error::Result {
        let _tvc = TvcClient::new(load_branch_name()?, StdFileSystem);
        // let status = tvc.merge(self.source_branch)?;
        // println!("merged status = {status:?}");
        // Ok(())
        todo!();
    }
}
