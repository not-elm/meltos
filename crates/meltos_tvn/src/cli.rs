mod init;

use auto_delegate::{delegate, Delegate};
use clap::Subcommand;
use crate::cli::init::InitArgs;
use crate::error;


#[delegate]
pub trait CommandExecutable{
    fn execute(self) -> error::Result;
}


#[derive(Subcommand, Debug, Delegate)]
#[to(CommandExecutable)]
pub enum TvnCommand{
    Init(InitArgs)
}