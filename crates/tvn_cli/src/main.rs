use std::env;

use clap::Parser;

use meltos::user::SessionId;

use crate::command::{CommandExecutable, Commands};

mod command;
mod error;

#[tokio::main]
async fn main() {
    let command = Commands::parse();
    match command {
        Commands::Open(open) => open.execute().await.unwrap(),
        Commands::Join(join) => join.execute().await.unwrap(),
        Commands::Fetch(fetch) => fetch.execute().await.unwrap(),
        Commands::Stage(stage) => stage.execute().await.unwrap(),
        Commands::Commit(commit) => commit.execute().await.unwrap(),
        Commands::Push(push) => push.execute().await.unwrap(),
        Commands::Merge(merge) => merge.execute().await.unwrap()
    }
}


#[allow(unused)]
fn read_session_id() -> error::Result<SessionId> {
    let session_id = env::var("SESSION_ID").map_err(|_| error::Error::SessionIdNotExists)?;
    Ok(SessionId(session_id))
}
