use thiserror::Error;
use tokio::sync::broadcast::error::SendError;
use tokio::task::JoinError;

use meltos::command::client::ClientCommand;
use meltos::command::server::ServerCommand;
use meltos::room::RoomId;
use meltos::user::UserId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Meltos(#[from] meltos::error::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error(transparent)]
    SendServerOrder(#[from] SendError<ServerCommand>),

    #[error(transparent)]
    SendClientOrder(#[from] SendError<ClientCommand>),

    #[error("user_id {0} was already joined in session {1}")]
    RoomJoin(UserId, RoomId),

    #[error("websocket has been disconnected")]
    Disconnected,

    #[error("session not exists: session id = {0}")]
    SessionNotExists(RoomId),

    #[error("websocket message invalid command")]
    InvalidOrder,
}


impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.to_string()
    }
}


unsafe impl Send for Error {}
