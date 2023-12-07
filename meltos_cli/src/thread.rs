mod new;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ThreadCommands {
    New(new::New),
}

impl ThreadCommands {
    pub fn run(self) {
        match self {
            ThreadCommands::New(new) => new.run(),
        }
    }
}
