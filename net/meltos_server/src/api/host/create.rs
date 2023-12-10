use axum::extract::State;

use meltos::session::SessionId;

use crate::effect::create_effect;
use crate::HttpResult;
use crate::state::Rooms;

pub async fn create(State(rooms): State<Rooms>) -> HttpResult<String> {
    let session_id = SessionId("session".to_string());
    rooms
        .lock()
        .await
        .insert(session_id.clone(), create_effect(30));
    Ok(session_id.to_string())
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
