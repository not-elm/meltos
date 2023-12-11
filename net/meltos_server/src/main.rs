use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use std::net::SocketAddr;

use meltos_util::tracing::tracing_init;

use crate::state::AppState;

mod api;
mod error;
mod room;
mod state;


pub type HttpResult<T> = std::result::Result<T, StatusCode>;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_init();

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).await?;

    axum::serve(listener, app()).await?;
    Ok(())
}


fn app() -> Router {
    Router::new()
        .route("/room/open", post(api::room::open))
        .route("/room/connect", get(api::room::connect))
        .with_state(AppState::default())
}
