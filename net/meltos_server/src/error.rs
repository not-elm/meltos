use thiserror::Error;
use tokio::sync::broadcast::error::SendError;
use tokio::task::JoinError;
use meltos::command::client::ClientOrder;

use meltos::session::RoomId;
use meltos::user::UserId;

use meltos::command::server::ServerCommand;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Meltos(#[from] meltos::error::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error(transparent)]
    SendServerOrder(#[from] SendError<ServerCommand>),

    #[error(transparent)]
    SendClientOrder(#[from] SendError<ClientOrder>),

    #[error("user_id {0} was already joined in session {1}")]
    RoomJoin(UserId, RoomId),

    #[error("websocket has been disconnected")]
    Disconnected,

    #[error("session not exists: session id = {0}")]
    SessionNotExists(RoomId),

    #[error("websocket message invalid command")]
    InvalidOrder,

    #[error("failed serialize to binary")]
    SerializeToBinary,
}


impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.to_string()
    }
}


unsafe impl Send for Error {}
