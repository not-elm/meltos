use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;

use meltos::room::RoomId;

use crate::error::Error;
use crate::room::create_room;
use crate::state::Rooms;


#[tracing::instrument]
pub async fn open(State(rooms): State<Rooms>) -> Response {
    let room_id = RoomId("session".to_string());
    match create_room(rooms, room_id.clone(), 30).await {
        Ok(()) => response_success_create_room(room_id),
        Err(error) => response_error_already_exists_room(error),
    }
}


fn response_success_create_room(room_id: RoomId) -> Response {
    Response::builder()
        .body(Body::from(room_id.to_string()))
        .unwrap()
}


fn response_error_already_exists_room(error: Error) -> Response {
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
            .uri("/room/open")
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
