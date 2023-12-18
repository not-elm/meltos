use async_trait::async_trait;
use clap::Args;

use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::operation::commit::Commit;

use crate::command::CommandExecutable;

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
