use clap::Args;

use crate::cli::CommandExecutable;

#[derive(Args, Debug, Clone, Eq, PartialEq)]
pub struct InitArgs {
    path: String,
}


impl CommandExecutable for InitArgs {
    fn execute(self) -> crate::error::Result {
        todo!()
    }
}


