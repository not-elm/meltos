use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::routing::{get, post};
use tower_http::decompression::RequestDecompressionLayer;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use meltos_backend::discussion::global::sqlite::SqliteDiscussionIo;
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_backend::session::sqlite::SqliteSessionIo;

use crate::state::AppState;
use crate::state::config::AppConfigs;

mod api;
mod channel;
mod error;
mod middleware;
mod room;
mod state;

fn tracing_init() {
    // let info_writer = tracing_appender::rolling::minutely("./log/info", "info.log");
    // let error_writer = tracing_appender::rolling::minutely("./log/error", "error.log");
    //
    // let info_layer = tracing_subscriber::fmt::Layer::default()
    //     .with_ansi(false)
    //     .with_writer(info_writer.with_max_level(Level::INFO));
    // let error_layer = tracing_subscriber::fmt::Layer::default()
    //     .with_ansi(false)
    //     .with_writer(error_writer.with_max_level(Level::WARN));
    // let console_layer = tracing_subscriber::fmt::Layer::default()
    //     .with_ansi(false)
    //     .with_writer(console_subscriber::ConsoleLayer::new());
    //
    // tracing_subscriber::registry()
    //     .with(console_layer)
    //     .with(info_layer)
    //     .with(error_layer)
    //     .init();
    console_subscriber::init();
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "DEBUG");
    tracing_init();

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 3000))).await?;

    axum::serve(listener, app::<SqliteSessionIo, SqliteDiscussionIo>()).await?;
    Ok(())
}

fn app<Session, Discussion>() -> Router
    where
        Session: SessionIo + NewSessionIo + Debug + 'static
{
    Router::new()
        .route("/room/open", post(api::room::open::<Session>))
        .layer(DefaultBodyLimit::max(bundle_request_body_size()))
        .nest("/room/:room_id", room_operations_router())
        .with_state(AppState::new())
        .layer(RequestDecompressionLayer::new())
}

fn room_operations_router() -> Router<AppState> {
    Router::new()
        .route("/channel", get(api::room::channel))
        .route("/join", post(api::room::join))
        .route("/request", post(api::room::request))
}


#[inline(always)]
fn bundle_request_body_size() -> usize {
    // Bundleの最大サイズは100MIBに設定したいですが、json形式でデータが送られてくる関係上
    // リクエストボディのデータサイズが大きくなることを考慮して4倍までは許容するように
    // 今後修正する可能性あり
    AppConfigs::default().limit_tvc_repository_size * 4
}
