use std::fmt::Debug;
use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post};
use meltos_backend::user::mock::MockUserSessionIo;

use meltos_backend::user::UserSessionIo;
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

    axum::serve(listener, app(MockUserSessionIo::default())).await?;
    Ok(())
}


fn app<Session>(session: Session) -> Router
    where Session: UserSessionIo + Debug + Clone + 'static
{
    Router::new()
        .route("/room/open", post(api::room::open::<Session>))
        .route("/room/connect", get(api::room::connect))
        .with_state(AppState::<Session>::new(session))
}
