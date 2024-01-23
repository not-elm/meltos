use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use thiserror::Error;

use meltos::discussion::id::DiscussionId;
use meltos::schema::error::{ErrorResponseBodyBase, ExceedBundleSizeBody};

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("failed discussion io message: {0}")]
    FailedDiscussionIo(String),

    #[error("bundle size to exceed; actual_bundle_size: {actual_bundle_size}, limit_bundle_size: {limit_bundle_size}")]
    ExceedBundleSize {
        actual_bundle_size: usize,
        limit_bundle_size: usize,
    },

    #[error(transparent)]
    Axum(#[from] axum::Error),

    #[error("failed create room message: {0}")]
    FailedCreateDiscussionIo(String),

    #[error("failed create session io message: {0}")]
    FailedCreateSessionIo(String),

    #[error("discussion id = {0} is not exists")]
    DiscussionNotExists(DiscussionId),

    #[error("failed to send channel message : {0}")]
    FailedSendChannelMessage(String),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ExceedBundleSize { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            Error::DiscussionNotExists(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_type(&self) -> &str {
        match self {
            Error::ExceedBundleSize { .. } => "tvc",
            _ => "unknown"
        }
    }

    fn as_body_base(&self) -> ErrorResponseBodyBase {
        ErrorResponseBodyBase {
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
        let status_code = e.status_code();
        Response::builder()
            .status(status_code)
            .body(Body::new(e.into_body()))
            .unwrap()
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