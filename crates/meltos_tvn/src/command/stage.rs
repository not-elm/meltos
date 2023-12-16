use async_trait::async_trait;
use clap::Args;
use crate::branch::BranchName;

use crate::command::CommandExecutable;
use crate::file_system::file::StdFileSystem;
use crate::operation::stage::Stage;

#[derive(Args, Clone, Debug)]
pub struct StageArgs {
    workspace_path: String,
}

#[async_trait]
impl CommandExecutable for StageArgs {
    async fn execute(self) -> crate::error::Result {
        let stage = Stage::new(BranchName::working(StdFileSystem)?, StdFileSystem);
        stage.execute(&self.workspace_path)
    }
}