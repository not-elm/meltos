use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;

use meltos_util::tracing::tracing_init;

use crate::api::host;
use crate::state::AppState;

mod api;
mod error;
mod room;
mod state;


pub type HttpResult<T> = std::result::Result<T, StatusCode>;

#[tokio::main]
async fn main() {
    tracing_init();

    let _app = app();
}


fn app() -> Router {
    Router::new()
        .route("/host/create", post(host::create))
        .route("/host/connect", get(host::connect))
        .with_state(AppState::default())
}
