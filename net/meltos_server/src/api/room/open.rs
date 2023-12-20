use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use meltos::room::RoomId;
use meltos::schema::request::room::Open;
use meltos::schema::response::room::Opened;
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
    room.save_bundle(param.bundle)?;
    rooms.insert_room(room).await;

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
    use tower::ServiceExt;

    use meltos::schema::response::room::Opened;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::test_util::{open_room_request, ResponseConvertable};
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
}
