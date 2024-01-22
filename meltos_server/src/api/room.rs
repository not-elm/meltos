use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
pub use channel::channel;
pub use join::join;
pub use leave::leave;
pub use open::open;
use serde_json::json;
pub use sync::sync;

pub mod discussion;
pub mod tvc;

mod channel;

mod join;
mod leave;
mod open;
mod sync;

pub(crate) fn response_error_exceed_bundle_size(
    actual_bundle_size: usize,
    limit_bundle_size: usize,
) -> Response {
    Response::builder()
        .status(StatusCode::PAYLOAD_TOO_LARGE)
        .body(Body::from(
            json!({
                "message" : "bundle size to exceed",
                "actual_bundle_size": actual_bundle_size,
                "limit_bundle_size" : limit_bundle_size,
            })
            .to_string(),
        ))
        .unwrap()
}
