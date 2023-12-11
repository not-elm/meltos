use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;

use meltos::room::RoomId;
use meltos::user::UserToken;
use meltos_backend::user::UserSessionIo;

use crate::room::{Room, Rooms};
use crate::state::SessionState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenParam {
    user_token: UserToken,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenResponse {
    room_id: RoomId,
}


#[tracing::instrument]
pub async fn open<Session: UserSessionIo + Clone + Debug>(
    State(rooms): State<Rooms>,
    State(session): State<SessionState<Session>>,
    Json(param): Json<OpenParam>,
) -> Response {
    match session.fetch_user_id(param.user_token).await {
        Ok(user_id) => {
            let room = Room::open(user_id, 30);
            let room_id = room.id.clone();
            rooms.insert_room(room).await;
            response_success_create_room(room_id)
        }
        Err(_) => response_error_not_exists_user_id()
    }
}


fn response_success_create_room(room_id: RoomId) -> Response {
    Response::builder()
        .body(Body::from(serde_json::to_string(&OpenResponse {
            room_id
        }).unwrap()))
        .unwrap()
}


fn response_error_not_exists_user_id() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(
            json!({
                "error" : "user id not exists"
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

    use meltos::user::{UserId, UserToken};
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::UserSessionIo;

    use crate::{app, error};
    use crate::api::room::open::{OpenParam, OpenResponse};

    #[tokio::test]
    async fn failed_if_not_logged_in() {
        let app = app(MockUserSessionIo::default());
        let response = app.oneshot(request(UserToken("token".to_string()))).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }


    #[tokio::test]
    async fn success_if_logged_in() -> error::Result {
        let session = MockUserSessionIo::default();
        session.register(UserToken("token".to_string()), UserId::from("user")).await.unwrap();

        let app = app(session);
        let response = app.oneshot(request(UserToken("token".to_string()))).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_slice::<OpenResponse>(&response.into_body().collect().await.unwrap().to_bytes())?;

        Ok(())
    }


    fn request(user_token: UserToken) -> Request {
        Request::builder()
            .method(http::Method::POST)
            .header("Content-Type", "application/json")
            .uri("/room/open")
            .body(Body::from(serde_json::to_string(&OpenParam {
                user_token
            }).unwrap()))
            .unwrap()
    }
}
