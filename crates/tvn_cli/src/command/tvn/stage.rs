use async_trait::async_trait;
use clap::Args;
use meltos_tvn::branch::BranchName;
use meltos_tvn::file_system::file::StdFileSystem;
use meltos_tvn::operation::stage::Stage;

use crate::command::CommandExecutable;

#[derive(Args, Clone, Debug)]
pub struct StageArgs {
    workspace_path: String,
}

#[async_trait]
impl CommandExecutable for StageArgs {
    async fn execute(self) -> crate::error::Result {
        let stage = Stage::new(BranchName::working(StdFileSystem)?, StdFileSystem);
        stage.execute(&self.workspace_path)?;
        Ok(())
    }
}
