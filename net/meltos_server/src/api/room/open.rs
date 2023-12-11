use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use meltos::command::client::room::Opened;
use meltos::command::request::room::Open;
use meltos::room::RoomId;
use meltos_backend::user::SessionIo;
use meltos_util::serde::SerializeJson;

use crate::api::HttpResult;
use crate::room::{Room, Rooms};
use crate::state::SessionState;

#[tracing::instrument]
pub async fn open<Session: SessionIo + Clone + Debug>(
    State(rooms): State<Rooms>,
    State(session): State<SessionState<Session>>,
    Json(param): Json<Open>,
) -> HttpResult {
    let user_id = session.try_fetch_user_id(param.user_token).await?;
    let room = Room::open(user_id);
    let room_id = room.id.clone();
    rooms.insert_room(room).await;
    Ok(response_success_create_room(room_id))
}


fn response_success_create_room(room_id: RoomId) -> Response {
    Response::builder()
        .body(Body::from(Opened { room_id }.as_json()))
        .unwrap()
}


#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use meltos::command::client::room::Opened;
    use meltos::user::{UserId, UserToken};
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;

    use crate::api::test_util::open_room_request;
    use crate::{app, error};

    #[tokio::test]
    async fn failed_if_not_logged_in() {
        let app = app(MockUserSessionIo::default());
        let response = app
            .oneshot(open_room_request(UserToken("token".to_string())))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }


    #[tokio::test]
    async fn success_if_logged_in() -> error::Result {
        let session = MockUserSessionIo::default();
        session
            .register(UserToken("token".to_string()), UserId::from("user"))
            .await
            .unwrap();

        let app = app(session);
        let response = app
            .oneshot(open_room_request(UserToken("token".to_string())))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_slice::<Opened>(
            &response.into_body().collect().await.unwrap().to_bytes(),
        )?;

        Ok(())
    }
}
