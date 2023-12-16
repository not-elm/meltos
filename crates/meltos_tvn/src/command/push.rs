use async_trait::async_trait;
use clap::Args;

use crate::branch::BranchName;
use crate::command::CommandExecutable;
use crate::file_system::file::StdFileSystem;
use crate::operation::push::Push;
use crate::remote_client::mock::MockRemoteClient;

#[derive(Args, Clone, Debug)]
pub struct PushArgs {
    commit_text: String,
}


#[async_trait]
impl CommandExecutable for PushArgs {
    async fn execute(self) -> crate::error::Result {
        let push = Push::new(BranchName::working(StdFileSystem)?, StdFileSystem);
        push.execute(&MockRemoteClient::default()).await?;
        Ok(())
    }
}
