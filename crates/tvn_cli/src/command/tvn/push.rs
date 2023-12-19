use async_trait::async_trait;
use clap::Args;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::operation::push::Push;

use crate::command::CommandExecutable;


#[derive(Args, Clone, Debug)]
pub struct PushArgs {
    commit_text: String,
}


#[async_trait]
impl CommandExecutable for PushArgs {
    async fn execute(self) -> crate::error::Result {
        let push = Push::new(BranchName::working(StdFileSystem)?, StdFileSystem);
        // push.execute(&mut LocalHttpClient::new()).await?;
        Ok(())
    }
}
