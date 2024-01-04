use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use meltos::room::RoomId;
use meltos::schema::room::Open;
use meltos::schema::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_backend::discussion::DiscussionIo;
use meltos_backend::user::SessionIo;
use meltos_util::serde::SerializeJson;

use crate::api::HttpResult;
use crate::room::{Room, Rooms};
use crate::state::SessionState;

#[tracing::instrument]
pub async fn open<Session, Discussion>(
    State(rooms): State<Rooms>,
    State(session): State<SessionState<Session>>,
    Json(param): Json<Open>,
) -> HttpResult
where
    Discussion: DiscussionIo + Default + 'static,
    Session: SessionIo + Debug,
{
    let (user_id, session_id) = session.register(param.user_id.clone()).await?;
    let room = Room::open::<Discussion>(user_id.clone());
    let room_id = room.id.clone();
    let life_time = param.life_time_duration();
    if let Some(bundle) = param.bundle {
        room.save_bundle(bundle)?;
    }

    rooms.insert_room(room, life_time).await;

    Ok(response_success_create_room(room_id, user_id, session_id))
}

fn response_success_create_room(
    room_id: RoomId,
    user_id: UserId,
    session_id: SessionId,
) -> Response {
    Response::builder()
        .body(Body::from(
            Opened {
                room_id,
                user_id,
                session_id,
            }
            .as_json(),
        ))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use std::time::Duration;
    use tower::ServiceExt;

    use meltos::schema::room::Opened;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_tvc::file_system::mock::MockFileSystem;

    use crate::api::test_util::{
        create_discussion_request, http_call, open_room_request, open_room_request_with_options,
        ResponseConvertable,
    };
    use crate::{app, error};

    #[tokio::test]
    async fn return_room_id_and_session_id() -> error::Result {
        let app = app(
            MockUserSessionIo::default(),
            MockGlobalDiscussionIo::default(),
        );
        let mock = MockFileSystem::default();
        let response = app.oneshot(open_room_request(mock)).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.room_id.0.len(), 40);
        assert_eq!(opened.session_id.0.len(), 40);
        Ok(())
    }

    #[tokio::test]
    async fn timeout() -> error::Result {
        let mut app = app(
            MockUserSessionIo::default(),
            MockGlobalDiscussionIo::default(),
        );
        let response = http_call(&mut app, open_room_request_with_options(None, Some(1))).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let opened = response.deserialize::<Opened>().await;
        let response = app
            .oneshot(create_discussion_request(
                "title".to_string(),
                opened.room_id,
                &opened.session_id,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }
}
