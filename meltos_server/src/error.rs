use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use strum::AsRefStr;
use thiserror::Error;

use meltos::room::RoomId;
use meltos::schema::error::{ErrorResponseBodyBase, ExceedBundleSizeBody, ReachedCapacityBody};
use meltos::user::UserId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug, AsRefStr)]
pub enum Error {
    #[error("room not exists; room_id: {0}")]
    RoomNotExists(RoomId),

    #[error("room owner disconnected room_id: {0}")]
    RoomOwnerDisconnected(RoomId),

    #[error("user not exists room_id: {0}, user_id:{1}")]
    UserNotExists(RoomId, UserId),

    #[error("reached capacity; capacity: {0}")]
    ReachedCapacity(u64),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("bundle size to exceed; actual_bundle_size: {actual_bundle_size}, limit_bundle_size: {limit_bundle_size}")]
    ExceedBundleSize {
        actual_bundle_size: usize,
        limit_bundle_size: usize,
    },

    #[error("failed to send channel message : {0}")]
    FailedSendChannelMessage(String),

    #[error("failed create session : {0}")]
    FailedCreateSessionIo(String),

    #[error(transparent)]
    Background(#[from] meltos_backend::error::Error)
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Background(e) => e.status_code(),
            Error::ReachedCapacity(_) => StatusCode::TOO_MANY_REQUESTS,
            Error::RoomNotExists(_) => StatusCode::NOT_FOUND,
            Error::ExceedBundleSize { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn category(&self) -> &str {
        match self {
            Error::Background(e) => e.category(),
            Error::RoomNotExists(_) | Error::ReachedCapacity(_) => "session",
            _ => "unknown"
        }
    }

    #[inline(always)]
    fn error_type(&self) -> &str {
        match self {
            Error::Background(e) => e.error_type(),
            _ => self.as_ref()
        }
    }

    fn as_body_base(&self) -> ErrorResponseBodyBase {
        ErrorResponseBodyBase {
            category: self.category().to_string(),
            error_type: self.error_type().to_string(),
            message: self.to_string(),
        }
    }

    fn into_body(self) -> String {
        let base = self.as_body_base();
        match self {
            Error::ExceedBundleSize { limit_bundle_size, actual_bundle_size } => {
                serde_json::to_string(&ExceedBundleSizeBody {
                    base,
                    limit_bundle_size,
                    actual_bundle_size,
                }).unwrap()
            }
            Error::ReachedCapacity(capacity) => {
                serde_json::to_string(&ReachedCapacityBody {
                    base,
                    capacity,
                }).unwrap()
            }
            _ => serde_json::to_string(&base).unwrap()
        }
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.to_string()
    }
}


impl From<Error> for Response {
    #[inline(always)]
    fn from(e: Error) -> Self {
        match e {
            _ => {
                let status_code = e.status_code();
                Response::builder()
                    .status(status_code)
                    .body(Body::new(e.into_body()))
                    .unwrap()
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::error::Error;

    #[test]
    fn status_code_is_payload_too_large() {
        assert_eq!(Error::ExceedBundleSize {
            limit_bundle_size: 100,
            actual_bundle_size: 101,
        }.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}