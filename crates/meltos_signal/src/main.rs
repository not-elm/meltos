use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;
use http::StatusCode;

use meltos_util::tracing;
use offer::connect;

use crate::session::mock::MockSessionIo;
use crate::session::SessionIo;
use crate::state::AppState;

mod error;
mod offer;
mod session;
mod shared;
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
        .route("/offer/init", post(offer::init::init::<S>))
        .route("/offer/connect", get(connect::connect))
        .with_state(AppState::<S>::default())
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::app;
    use crate::offer::init::OfferParam;
    use crate::session::mock::MockSessionIo;
    use crate::session::SessionId;

    #[tokio::test]
    async fn offer() {
        let app = app::<MockSessionIo>();
        let offer = OfferParam::default();
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/offer/init")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&offer).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            SessionId::from(&offer.session_description)
                .to_string()
                .as_bytes()
        );
    }
}
