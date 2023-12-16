use async_trait::async_trait;
use auto_delegate::Delegate;
use clap::{Parser, Subcommand};

use crate::command::commit::CommitArgs;
use crate::command::init::InitArgs;
use crate::command::push::PushArgs;
use crate::command::stage::StageArgs;
use crate::error;

pub mod init;
pub mod stage;
pub mod commit;
pub mod push;

#[async_trait]
#[auto_delegate::delegate]
pub trait CommandExecutable {
    async fn execute(self) -> error::Result;
}


#[derive(Debug, Delegate, Parser)]
#[to(CommandExecutable)]
pub enum TvnCommand {
    Init(InitArgs),
    Stage(StageArgs),
    Commit(CommitArgs),
    Push(PushArgs),
}