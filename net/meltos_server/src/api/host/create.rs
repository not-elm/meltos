use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use tracing::debug;

use meltos::room::RoomId;

use crate::room::room_effect;
use crate::state::Rooms;

#[tracing::instrument]
pub async fn create(State(rooms): State<Rooms>) -> Response {
    debug!("create room");
    let room_id = RoomId("session".to_string());
    match room_effect(rooms, room_id.clone(), 30).await {
        Ok(()) => {
            Response::builder()
                .body(Body::from(room_id.to_string()))
                .unwrap()
        }
        Err(error) => {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(
                    json!({
                        "description" : error.to_string()
                    })
                        .to_string(),
                ))
                .unwrap()
        }
    }
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::app;

    #[tokio::test]
    async fn create() {
        let app = app();
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/host/create")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            "session".as_bytes()
        );
    }
}
