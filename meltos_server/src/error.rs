use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use thiserror::Error;
use tokio::task::JoinError;

use meltos::discussion::id::DiscussionId;
use meltos::room::RoomId;
use meltos::user::UserId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("failed discussion io message: {0}")]
    FailedDiscussionIo(String),
    
    #[error(transparent)]
    Meltos(#[from] meltos::error::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error("failed sent server schema error")]
    SendServerOrder,

    #[error("failed sent remote schema error")]
    SendClientOrder,

    #[error("room_id {0} was already created")]
    RoomAlreadyExists(RoomId),

    #[error("failed create room message: {0}")]
    FailedCreateDiscussionIo(String),

    #[error("failed create session io message: {0}")]
    FailedCreateSessionIo(String),

    #[error("user_id {0} was already joined in session {1}")]
    RoomJoin(UserId, RoomId),

    #[error("websocket has been disconnected")]
    Disconnected,

    #[error("discussion id = {0} is not exists")]
    DiscussionNotExists(DiscussionId),

    #[error("session not exists: session id = {0}")]
    SessionNotExists(RoomId),

    #[error("failed to send channel message : {0}")]
    FailedSendChannelMessage(String),

    #[error("websocket message invalid schema")]
    InvalidOrder,
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.to_string()
    }
}

impl From<Error> for Response {
    fn from(value: Error) -> Self {
        let status_code = match value {
            Error::DiscussionNotExists(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status_code)
            .body(Body::new(value.to_string()))
            .unwrap()
    }
}

unsafe impl Send for Error {}
