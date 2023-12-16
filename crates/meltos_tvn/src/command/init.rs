use async_trait::async_trait;
use clap::Args;

use crate::branch::BranchName;
use crate::command::CommandExecutable;
use crate::file_system::file::StdFileSystem;
use crate::operation;

#[derive(Args, Debug, Clone, Eq, PartialEq)]
pub struct InitArgs;



#[async_trait]
impl CommandExecutable for InitArgs {
    async fn execute(self) -> crate::error::Result {
        let init = operation::init::Init::new(BranchName::main(), StdFileSystem);
        init.execute()?;

        Ok(())
    }
}


