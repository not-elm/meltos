use std::fmt::Debug;
use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::Router;

use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
use meltos_backend::discussion::DiscussionIo;
use meltos_backend::user::mock::MockUserSessionIo;
use meltos_backend::user::SessionIo;
use meltos_util::tracing::tracing_init;

use crate::state::AppState;

mod api;
mod error;
mod middleware;
mod room;
mod state;


pub type HttpResult<T> = std::result::Result<T, StatusCode>;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_init();

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).await?;

    axum::serve(
        listener,
        app(
            MockUserSessionIo::default(),
            MockGlobalDiscussionIo::default(),
        ),
    )
    .await?;
    Ok(())
}


fn app<Session, Discussion>(session: Session, _: Discussion) -> Router
where
    Session: SessionIo + Debug + Clone + 'static,
    Discussion: DiscussionIo + Default + 'static,
{
    Router::new()
        .route("/room/open", post(api::room::open::<Discussion>))
        .route("/room/connect", get(api::room::connect))
        .nest("/room/:room_id", room_operations_router())
        .with_state(AppState::<Session>::new(session))
}


fn room_operations_router<Session>() -> Router<AppState<Session>>
where
    Session: SessionIo + Clone + Debug + 'static,
{
    Router::new().nest("/discussion/global", global_discussion_route())
}


fn global_discussion_route<Session>() -> Router<AppState<Session>>
where
    Session: SessionIo + Clone + Debug + 'static,
{
    Router::new()
        .route("/create", post(api::room::discussion::global::create))
        .route("/speak", post(api::room::discussion::global::speak))
        .route("/reply", post(api::room::discussion::global::reply))
        .route("/close", delete(api::room::discussion::global::close))
}
