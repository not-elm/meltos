use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use thiserror::Error;

use meltos::discussion::id::DiscussionId;
use meltos::room::RoomId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("session id not exists")]
    UserIdNotExists,

    #[error("discussion not exists id: {0}")]
    DiscussionNotExists(DiscussionId),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error("database already removed room_id: {0}")]
    DatabaseAlreadyRemoved(RoomId),
}


impl From<Error> for Response {
    fn from(e: Error) -> Self {
        let status_code = if matches!(e, Error::UserIdNotExists) {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        Response::builder()
            .status(status_code)
            .body(Body::from(json!({
                "error" : e.to_string()
            }).to_string()))
            .unwrap()
    }
}


unsafe impl Send for Error {}