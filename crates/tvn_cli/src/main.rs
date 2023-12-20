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
    }
}


fn read_session_id() -> error::Result<SessionId> {
    let session_id = env::var("SESSION_ID").map_err(|_| error::Error::SessionIdNotExists)?;
    Ok(SessionId(session_id))
}
