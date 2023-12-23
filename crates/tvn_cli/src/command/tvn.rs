use auto_delegate::Delegate;
use clap::Parser;

use commit::CommitArgs;
use init::InitArgs;
use push::PushArgs;
use stage::StageArgs;

pub mod commit;
pub mod fetch;
pub mod init;
pub mod merge;
pub mod push;
pub mod stage;

#[derive(Debug, Delegate, Parser)]
#[to(CommandExecutable)]
pub enum TvnCommand {
    Init(InitArgs),
    Stage(StageArgs),
    Commit(CommitArgs),
    Push(PushArgs),
}
