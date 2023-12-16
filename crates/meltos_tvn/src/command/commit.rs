use async_trait::async_trait;
use clap::Args;

use crate::branch::BranchName;
use crate::command::CommandExecutable;
use crate::file_system::file::StdFileSystem;
use crate::operation::commit::Commit;

#[derive(Args, Clone, Debug)]
pub struct CommitArgs {
    commit_text: String,
}


#[async_trait]
impl CommandExecutable for CommitArgs {
    async fn execute(self) -> crate::error::Result {
        let commit = Commit::new(BranchName::working(StdFileSystem)?, StdFileSystem);
        commit.execute(self.commit_text)?;
        Ok(())
    }
}