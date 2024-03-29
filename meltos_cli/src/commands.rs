use async_trait::async_trait;
use clap::Parser;

use crate::commands::all::AllArgs;
use meltos_client::config::SessionConfigs;
use meltos_tvc::branch::BranchName;

use crate::commands::commit::CommitArgs;
use crate::commands::fetch::FetchArgs;
use crate::commands::join::JoinArgs;
use crate::commands::leave::LeaveArgs;
use crate::commands::merge::MergeArgs;
use crate::commands::meta::MetaArgs;
use crate::commands::open::OpenArgs;
use crate::commands::push::PushArgs;
use crate::commands::reply::ReplyArgs;
use crate::commands::speak::SpeakArgs;
use crate::commands::stage::StageArgs;

mod all;
mod commit;
mod fetch;
mod join;
mod leave;
mod merge;
mod meta;
mod open;
mod push;
mod reply;
mod speak;
mod stage;

#[async_trait(?Send)]
pub trait CommandExecutable {
    /// execute cli command.
    async fn execute(self) -> meltos_client::error::Result;
}

#[derive(Parser, Debug, Clone)]
pub enum Commands {
    Open(OpenArgs),
    Join(JoinArgs),
    Leave(LeaveArgs),
    Stage(StageArgs),
    Fetch(FetchArgs),
    Commit(CommitArgs),
    Push(PushArgs),
    Merge(MergeArgs),
    Speak(SpeakArgs),
    Reply(ReplyArgs),
    All(AllArgs),
    Meta(MetaArgs),
}

#[async_trait(?Send)]
impl CommandExecutable for Commands {
    async fn execute(self) -> meltos_client::error::Result {
        match self {
            Self::Open(c) => c.execute().await,
            Self::Join(c) => c.execute().await,
            Self::Leave(c) => c.execute().await,
            Self::Fetch(c) => c.execute().await,
            Self::Stage(c) => c.execute().await,
            Self::Commit(c) => c.execute().await,
            Self::Push(c) => c.execute().await,
            Self::Merge(c) => c.execute().await,
            Self::Speak(c) => c.execute().await,
            Self::Reply(c) => c.execute().await,
            Self::All(c) => c.execute().await,
            Self::Meta(c) => c.execute().await,
        }
    }
}

const PATH: &str = "configs.json";

#[inline(always)]
fn load_branch_name() -> meltos_tvc::error::Result<BranchName> {
    Ok(BranchName(load_configs()?.user_id.0))
}

#[inline(always)]
fn load_configs() -> meltos_tvc::error::Result<SessionConfigs> {
    let buf = std::fs::read(PATH)?;
    Ok(serde_json::from_slice(&buf)?)
}

#[inline(always)]
fn save_configs(configs: &SessionConfigs) -> meltos_tvc::error::Result {
    std::fs::write(PATH, serde_json::to_string(configs).unwrap())?;
    Ok(())
}
