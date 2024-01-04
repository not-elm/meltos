use clap::Parser;

use crate::commands::{CommandExecutable, Commands};

mod commands;

#[tokio::main]
async fn main() {
    Commands::parse().execute().await.unwrap();
}



