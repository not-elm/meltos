use async_trait::async_trait;
use clap::Parser;

use crate::command::open::OpenArgs;
use crate::error;

mod open;
mod tvn;
mod join;

#[async_trait]

pub trait CommandExecutable {
    async fn execute(self) -> error::Result;
}

#[derive(Debug, Parser)]
pub enum Commands {
    Open(OpenArgs),
}
