use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use strum::AsRefStr;
use thiserror::Error;

use meltos::schema::error::{ErrorResponseBodyBase, ExceedBundleSizeBody, ExceedRepositorySizeBody, ReachedCapacityBody};

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug, AsRefStr)]
pub enum Error {
    #[error("server resources are exhausted")]
    FailedCreatedRoom,

    #[error("room not exists")]
    RoomNotExists,

    #[error("reached capacity; capacity: {0}")]
    ReachedCapacity(u64),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Backend(#[from] meltos_backend::error::Error),

    #[error("failed tvc; {0}")]
    Tvc(#[from] meltos_tvc::error::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("bundle size to exceed; actual_bundle_size: {actual_bundle_size}, limit_bundle_size: {limit_bundle_size}")]
    ExceedBundleSize {
        actual_bundle_size: usize,
        limit_bundle_size: usize,
    },

    #[error("tvc repository size to exceed; actual_size: {actual_size}, limit_size: {limit_size}")]
    ExceedRepositorySize {
        actual_size: usize,
        limit_size: usize,
    },

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error("failed create room message: {0}")]
    FailedCreateDiscussionIo(String),

    #[error("failed create session io message: {0}")]
    FailedCreateSessionIo(String),

    #[error("failed to send channel message : {0}")]
    FailedSendChannelMessage(String),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::FailedCreatedRoom => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ReachedCapacity(_) => StatusCode::TOO_MANY_REQUESTS,
            Error::RoomNotExists => StatusCode::NOT_FOUND,
            Error::Backend(e) => e.status_code(),
            Error::ExceedRepositorySize {..} | Error::ExceedBundleSize { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn category(&self) -> &str {
        match self {
            Error::FailedCreatedRoom | Error::RoomNotExists | Error::ReachedCapacity(_) => "session",
            Error::ExceedRepositorySize {..} | Error::ExceedBundleSize { .. } | Error::Tvc(_) => "tvc",
            Error::Backend(e) => e.category(),
            _ => "unknown"
        }
    }

    #[inline(always)]
    fn error_type(&self) -> &str {
        match self {
            Error::Backend(e) => e.error_type(),
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
            Error::ExceedRepositorySize {limit_size, actual_size} => {
                serde_json::to_string(&ExceedRepositorySizeBody{
                    base,
                    limit_tvc_repository_size: limit_size,
                    actual_size
                }).unwrap()
            }
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
            Error::Backend(e) => e.into(),
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

    #[test]
    fn status_code_is_internal_server_error_tvc() {
        assert_eq!(Error::Tvc(meltos_tvc::error::Error::InvalidWorkspaceObj("Tree".to_string())).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_type_is_session_id_not_exists() {
        assert_eq!(Error::Backend(meltos_backend::error::Error::SessionIdNotExists).error_type(), "SessionIdNotExists");
    }
}