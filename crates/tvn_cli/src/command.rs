use crate::command::join::JoinArgs;
use async_trait::async_trait;
use clap::Parser;

use crate::command::open::OpenArgs;
use crate::command::tvn::commit::CommitArgs;
use crate::command::tvn::fetch::FetchArgs;
use crate::command::tvn::merge::MergeArgs;
use crate::command::tvn::push::PushArgs;
use crate::command::tvn::stage::StageArgs;
use crate::error;

mod join;
mod open;
mod tvn;

#[async_trait]

pub trait CommandExecutable {
    async fn execute(self) -> error::Result;
}

#[derive(Debug, Parser)]
pub enum Commands {
    Open(OpenArgs),
    Join(JoinArgs),
    Fetch(FetchArgs),
    Stage(StageArgs),
    Commit(CommitArgs),
    Push(PushArgs),
    Merge(MergeArgs),
}
