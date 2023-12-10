use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;
use http::StatusCode;

use meltos_util::tracing;

use crate::session::mock::MockSessionIo;
use crate::session::SessionIo;
use crate::state::AppState;

mod api;
mod error;
mod session;
mod state;


pub type HttpResult<T> = Result<T, StatusCode>;

#[tokio::main]
async fn main() -> error::Result {
    tracing::tracing_init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let tcp = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(tcp, app::<MockSessionIo>()).await?;
    Ok(())
}


fn app<S>() -> Router
where
    S: SessionIo + Clone + Default + 'static,
{
    Router::new()
        .route("/host/init", post(api::webrtc::host::init::<S>))
        .route("/host/connect", get(api::webrtc::host::connect))
        .route("/user/join", get(api::webrtc::user::join))
        .with_state(AppState::<S>::default())
}
