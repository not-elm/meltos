use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;

use axum::Router;
use axum::routing::{delete, get, post};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
use meltos_backend::discussion::global::sqlite::SqliteDiscussionIo;
use meltos_backend::user::mock::MockUserSessionIo;
use meltos_backend::user::SessionIo;

use crate::state::AppState;

mod api;
mod channel;
mod error;
mod middleware;
mod room;
mod state;

pub fn tracing_init() {
    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "ERROR");
    tracing_init();
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 3000))).await?;

    axum::serve(
        listener,
        app::<MockUserSessionIo, SqliteDiscussionIo>(MockUserSessionIo::default()),
    )
        .await?;
    Ok(())
}

fn app<Session, Discussion>(session: Session) -> Router
    where
        Session: SessionIo + Debug + Clone + 'static,
        Discussion: DiscussionIo + NewDiscussIo + 'static,
{
    Router::new()
        .route("/room/open", post(api::room::open::<Session, Discussion>))
        .layer(tower_http::limit::RequestBodyLimitLayer::new(100 * 1000 * 1000))
        .route("/room/:room_id", delete(api::room::leave::<Session>))
        .nest("/room/:room_id", room_operations_router())
        .with_state(AppState::<Session>::new(session))
}

fn room_operations_router<Session>() -> Router<AppState<Session>>
    where
        Session: SessionIo + Clone + Debug + 'static,
{
    Router::new()
        .route("/channel", get(api::room::channel))
        .route("/join", post(api::room::join))
        .nest("/tvc", tvc_routes())
        .nest("/discussion/global", global_discussion_route())
}

fn tvc_routes<Session>() -> Router<AppState<Session>>
    where
        Session: SessionIo + Clone + Debug + 'static,
{
    Router::new()
        .route("/fetch", get(api::room::tvc::fetch))
        .route("/push", post(api::room::tvc::push))
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
