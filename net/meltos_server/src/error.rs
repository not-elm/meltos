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

    #[error(transparent)]
    Meltos(#[from] meltos::error::Error),

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error(transparent)]
    TaskJoin(#[from] JoinError),

    #[error("failed sent server command error")]
    SendServerOrder,

    #[error("failed sent client command error")]
    SendClientOrder,

    #[error("room_id {0} was already created")]
    RoomCreate(RoomId),

    #[error("user_id {0} was already joined in session {1}")]
    RoomJoin(UserId, RoomId),

    #[error("websocket has been disconnected")]
    Disconnected,

    #[error("discussion id = {0} is not exists")]
    DiscussionNotExists(DiscussionId),

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
