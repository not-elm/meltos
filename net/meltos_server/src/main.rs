use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;

use axum::Router;
use axum::routing::{delete, get, post};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use meltos_backend::discussion::DiscussionIo;
use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
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
    env::set_var("RUST_LOG", "DEBUG");
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
        .route("/room/open", post(api::room::open::<Session, Discussion>))
        .route("/room/channel", get(api::room::channel))
        .nest("/room/:room_id", room_operations_router())
        .with_state(AppState::<Session>::new(session))
}

fn room_operations_router<Session>() -> Router<AppState<Session>>
    where
        Session: SessionIo + Clone + Debug + 'static,
{
    Router::new()
        .route("/join", post(api::room::join))
        .nest("/tvn", tvn_routes())
        .nest("/discussion/global", global_discussion_route())
}

fn tvn_routes<Session>() -> Router<AppState<Session>>
    where
        Session: SessionIo + Clone + Debug + 'static,
{
    Router::new()
        .route("/fetch", get(api::room::tvn::fetch))
        .route("/push", post(api::room::tvn::push))
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
