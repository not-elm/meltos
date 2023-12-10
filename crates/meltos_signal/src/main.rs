use std::net::SocketAddr;

use axum::{Json, Router};
use axum::extract::State;
use axum::routing::{get, post};
use http::StatusCode;

use meltos_util::tracing;
use offer::connect;

use crate::offer::init::OfferParam;
use crate::session::{SessionId, SessionIo};
use crate::session::mock::MockSessionIo;
use crate::state::{AppState, SessionIoState};

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
        .route("/offer", post(offer::<S>))
        .route("/offer/connect", get(connect::connect))
        .with_state(AppState::<S>::default())
}


async fn offer<S>(
    State(session_io): State<SessionIoState<S>>,
    Json(param): Json<OfferParam>,
) -> HttpResult<String>
    where
        S: SessionIo + Clone,
{
    let session_id = SessionId::from(&param.session_description);
    let session_id_str = session_id.to_string();
    match session_io
        .insert(session_id, param.session_description)
        .await
    {
        Ok(()) => Ok(session_id_str),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
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
                    .uri("/offer")
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
