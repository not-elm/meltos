use axum::body::Body;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use tracing::debug;
use meltos::room::RoomId;
use crate::state::Rooms;

#[tracing::instrument]
pub async fn create(State(rooms): State<Rooms>) -> impl IntoResponse {
    debug!("create room");
    let room_id = RoomId("session".to_string());
    // rooms
    //     .lock()
    //     .await
    //     .insert(session_id.clone(), room_effect(session_id.clone(), 30));
    Response::builder()
        .body(Body::from(room_id.to_string()))
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
