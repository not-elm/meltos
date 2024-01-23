use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{Response};
use thiserror::Error;

use meltos::discussion::id::DiscussionId;
use meltos::room::RoomId;
use meltos::schema::error::ErrorResponseBodyBase;
use meltos::user::UserId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("session id not exists")]
    SessionIdNotExists,

    #[error("user id conflict; id: {0}")]
    UserIdConflict(UserId),

    #[error("The number of users in the room has reached its limit; limits: {0}")]
    ReachedNumberOfUsersLimits(usize),

    #[error("discussion not exists id: {0}")]
    DiscussionNotExists(DiscussionId),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error("database already removed room_id: {0}")]
    DatabaseAlreadyRemoved(RoomId),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::SessionIdNotExists => StatusCode::UNAUTHORIZED,
            Error::ReachedNumberOfUsersLimits(_) | Error::DiscussionNotExists(_) | Error::UserIdConflict(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<Error> for ErrorResponseBodyBase {
    #[inline]
    fn from(e: Error) -> ErrorResponseBodyBase {
        let error_type = match &e {
            Error::UserIdConflict(_) | Error::SessionIdNotExists | Error::ReachedNumberOfUsersLimits(_) => "session",
            Error::DatabaseAlreadyRemoved(_) | Error::Io(_) | Error::Sqlite(_) => "io",
            Error::DiscussionNotExists(_) => "discussion"
        };
        ErrorResponseBodyBase {
            error_type: error_type.to_string(),
            message: e.to_string(),
        }
    }
}


impl From<Error> for Response {
    fn from(e: Error) -> Self {
        let status_code = e.status_code();
        let error_response: ErrorResponseBodyBase = e.into();
        Response::builder()
            .status(status_code)
            .body(Body::from(serde_json::to_string(&error_response).unwrap()))
            .unwrap()
    }
}


#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use meltos::discussion::id::DiscussionId;
    use meltos::user::UserId;

    use crate::error;

    #[test]
    fn it_return_unauthorized_code_if_session_id_not_exists() {
        assert_eq!(error::Error::SessionIdNotExists.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn it_return_bad_request_code_if_user_id_conflicts() {
        assert_eq!(error::Error::UserIdConflict(UserId::from("guest1")).status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn it_return_bad_request_code_if_discussion_id_not_exists() {
        assert_eq!(error::Error::DiscussionNotExists(DiscussionId("discussion_id".to_string())).status_code(), StatusCode::BAD_REQUEST);
    }
}
