use meltos::order::ServerOrder;
use meltos::session::SessionId;
use meltos::user::UserId;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;
use tokio::task::JoinError;

pub type Result<T: Send = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Meltos(#[from] meltos::error::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error(transparent)]
    Send(#[from] SendError<ServerOrder>),

    #[error("user_id {0} was already joined in session {1}")]
    RoomJoin(UserId, SessionId),

    #[error("websocket has been disconnected")]
    Disconnected,

    #[error("session not exists: session id = {0}")]
    SessionNotExists(SessionId),

    #[error("websocket message invalid order")]
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
