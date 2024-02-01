use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use strum::AsRefStr;
use thiserror::Error;

use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::MessageId;
use meltos::room::RoomId;
use meltos::schema::error::{DiscussionNotExistsBody, ErrorResponseBodyBase, MessageNotExistsBody};
use meltos::user::UserId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug, AsRefStr)]
pub enum Error {
    #[error("session id not exists")]
    SessionIdNotExists,

    #[error("user id conflict; id: {0}")]
    UserIdConflict(UserId),

    #[error("The number of users in the room has reached its limit; limits: {0}")]
    ReachedNumberOfUsersLimits(usize),

    #[error("discussion not exists; id: {0}")]
    DiscussionNotExists(DiscussionId),

    #[error("message not exists; id: {0}")]
    MessageNotExists(MessageId),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error("database already removed room_id: {0}")]
    DatabaseAlreadyRemoved(RoomId),
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::SessionIdNotExists => StatusCode::UNAUTHORIZED,
            Error::ReachedNumberOfUsersLimits(_)
            | Error::DiscussionNotExists(_)
            | Error::UserIdConflict(_)
            | Error::MessageNotExists(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline(always)]
    pub fn category(&self) -> &str {
        match self {
            Error::UserIdConflict(_)
            | Error::SessionIdNotExists
            | Error::ReachedNumberOfUsersLimits(_) => "session",
            Error::DatabaseAlreadyRemoved(_) | Error::Io(_) | Error::Sqlite(_) => "io",
            Error::DiscussionNotExists(_) | Error::MessageNotExists(_) => "discussion",
        }
    }

    #[inline(always)]
    pub fn error_name(&self) -> &str {
        self.as_ref()
    }

    pub fn into_body(self) -> String {
        let base = self.body_base();
        match self {
            Error::DiscussionNotExists(discussion_id) => {
                serde_json::to_string(&DiscussionNotExistsBody {
                    base,
                    discussion_id,
                })
                .unwrap()
            }
            Error::MessageNotExists(message_id) => {
                serde_json::to_string(&MessageNotExistsBody {
                    base,
                    message_id,
                })
                .unwrap()
            }
            _ => serde_json::to_string(&base).unwrap(),
        }
    }

    #[inline(always)]
    fn body_base(&self) -> ErrorResponseBodyBase {
        ErrorResponseBodyBase {
            category: self.category().to_string(),
            error_name: self.error_name().to_string(),
            message: self.to_string(),
        }
    }
}

impl From<Error> for Response {
    fn from(e: Error) -> Self {
        let status_code = e.status_code();
        Response::builder()
            .status(status_code)
            .body(Body::from(e.into_body()))
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
        assert_eq!(
            error::Error::SessionIdNotExists.status_code(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn it_return_bad_request_code_if_user_id_conflicts() {
        assert_eq!(
            error::Error::UserIdConflict(UserId::from("guest1")).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn it_return_bad_request_code_if_discussion_id_not_exists() {
        assert_eq!(
            error::Error::DiscussionNotExists(DiscussionId("discussion_id".to_string()))
                .status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn error_type_is_discussion_not_exists() {
        assert_eq!(
            error::Error::DiscussionNotExists(DiscussionId("discussion_id".to_string()))
                .error_name(),
            "DiscussionNotExists"
        );
    }
}
