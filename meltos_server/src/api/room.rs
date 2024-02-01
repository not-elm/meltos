use axum::response::Response;

pub use channel::channel;
pub use join::join;
pub use leave::leave;
pub use open::open;
pub use sync::sync;
pub use kick::kick;

pub mod discussion;
pub mod tvc;

mod channel;

mod join;
mod leave;
mod open;
mod sync;
mod kick;


#[inline(always)]
pub(crate) fn response_error_exceed_bundle_size(
    actual_bundle_size: usize,
    limit_bundle_size: usize,
) -> Response {
    crate::error::Error::ExceedBundleSize {
        actual_bundle_size,
        limit_bundle_size,
    }.into()
    // Response::builder()
    //     .status(StatusCode::PAYLOAD_TOO_LARGE)
    //     .body(Body::from(
    //         json!({
    //             "error_type": "tvc",
    //             "message" : format!("bundle size to exceed; actual_bundle_size: {actual_bundle_size}, limit_bundle_size: {limit_bundle_size}"),
    //             "actual_bundle_size": actual_bundle_size,
    //             "limit_bundle_size" : limit_bundle_size,
    //         })
    //             .to_string(),
    //     ))
    //     .unwrap()
}
