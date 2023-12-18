use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use meltos::command::client::room::Opened;
use meltos::room::RoomId;
use meltos_backend::discussion::DiscussionIo;
use meltos_tvn::operation::push::PushParam;
use meltos_util::serde::SerializeJson;

use crate::api::HttpResult;
use crate::middleware::user::SessionUser;
use crate::room::{Room, Rooms};

#[tracing::instrument]
pub async fn open<Discussion: DiscussionIo + Default + 'static>(
    State(rooms): State<Rooms>,
    SessionUser(user_id): SessionUser,
    Json(param): Json<PushParam>,
) -> HttpResult {
    let room = Room::open::<Discussion>(user_id);
    let room_id = room.id.clone();
    room.save_commits(param)?;
    rooms.insert_room(room).await;

    Ok(response_success_create_room(room_id))
}


fn response_success_create_room(room_id: RoomId) -> Response {
    Response::builder()
        .body(Body::from(
            Opened {
                room_id,
            }
            .as_json(),
        ))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use meltos::command::client::room::Opened;
    use meltos::user::UserId;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;
    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::test_util::{mock_session_id, open_room_request};
    use crate::{app, error};

    #[tokio::test]
    async fn failed_if_not_logged_in() {
        let app = app(
            MockUserSessionIo::default(),
            MockGlobalDiscussionIo::default(),
        );
        let mock = MockFileSystem::default();
        let response = app
            .oneshot(open_room_request(mock_session_id(), mock))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn success_if_logged_in() -> error::Result {
        let session = MockUserSessionIo::default();
        session
            .register(mock_session_id(), UserId::from("user"))
            .await
            .unwrap();

        let app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        let response = app
            .oneshot(open_room_request(mock_session_id(), mock))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_slice::<Opened>(
            &response.into_body().collect().await.unwrap().to_bytes(),
        )?;

        Ok(())
    }
}
