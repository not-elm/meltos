use std::fmt::Debug;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::extract::DefaultBodyLimit;
use axum::http::header::CONTENT_TYPE;
use axum::http::Method;
use axum::Router;
use axum::routing::{delete, get, post};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::cors::{Any, CorsLayer};
use tower_http::decompression::RequestDecompressionLayer;

use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
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
    //
    // tracing_subscriber::registry()
    //     .with(console_subscriber::spawn())
    //     .with(info_layer)
    //     .with(error_layer)
    //     .init();
    console_subscriber::init()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_init();
    let config_dir = PathBuf::from("/etc")
        .join("letsencrypt")
        .join("archive")
        .join("room.meltos.net");

    let config = RustlsConfig::from_pem_file(
        config_dir.join("cert1.pem"),
        config_dir.join("privkey1.pem"),
    )
        .await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    axum_server::bind_rustls(addr, config)
        .serve(app::<SqliteSessionIo, SqliteDiscussionIo>().into_make_service())
        .await?;

    Ok(())
}

fn app<Session, Discussion>() -> Router
    where
        Session: SessionIo + NewSessionIo + Debug + 'static,
        Discussion: DiscussionIo + NewDiscussIo + Debug + 'static,
{
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/room/open", post(api::room::open::<Session, Discussion>))
        .layer(DefaultBodyLimit::max(bundle_request_body_size()))
        .route("/room/:room_id", get(api::room::sync))
        .route("/room/:room_id", delete(api::room::leave))
        .nest("/room/:room_id", room_operations_router())
        .with_state(AppState::new())
        .layer(RequestDecompressionLayer::new())
        .layer(cors)
}

fn room_operations_router() -> Router<AppState> {
    Router::new()
        .route("/kick", post(api::room::kick))
        .route("/channel", get(api::room::channel))
        .route("/join", post(api::room::join))
        .nest("/tvc", tvc_routes())
        .nest("/discussion/global", global_discussion_route())
}

fn tvc_routes() -> Router<AppState> {
    Router::new()
        .route("/push", post(api::room::tvc::push))
        .layer(DefaultBodyLimit::max(bundle_request_body_size()))
        .route("/fetch", get(api::room::tvc::fetch))
}

fn global_discussion_route() -> Router<AppState> {
    Router::new()
        .route("/create", post(api::room::discussion::global::create))
        .route("/speak", post(api::room::discussion::global::speak))
        .route("/reply", post(api::room::discussion::global::reply))
        .route("/close", delete(api::room::discussion::global::close))
}

#[inline(always)]
fn bundle_request_body_size() -> usize {
    // Bundleの最大サイズは100MIBに設定したいですが、json形式でデータが送られてくる関係上
    // リクエストボディのデータサイズが大きくなることを考慮して4倍までは許容するように
    // 今後修正する可能性あり
    AppConfigs::default().limit_tvc_repository_size * 4
}
