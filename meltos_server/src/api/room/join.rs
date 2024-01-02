use std::fmt::Debug;

use axum::extract::State;
use axum::Json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::schema::room::{Join, Joined};
use meltos_backend::user::SessionIo;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::state::SessionState;

/// RoomIdに対応するRoomに参加
/// Roomが存在しない場合はstatus code404が返される
///
/// レスポンスはRoomのメタデータ
///
pub async fn join<Session: SessionIo + Debug>(
    State(session): State<SessionState<Session>>,
    SessionRoom(room): SessionRoom,
    Json(join): Json<Join>,
) -> HttpResult {
    let (user_id, session_id) = session.register(join.user_id).await?;
    let bundle = room.create_bundle()?;
    let joined = Joined {
        user_id: user_id.clone(),
        session_id,
        bundle,
    };
    room.send_all_users(ChannelMessage {
        message: MessageData::Joined {
            user_id: user_id.to_string(),
        },
        from: user_id,
    })
    .await?;

    Ok(joined.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use meltos::room::RoomId;
    use meltos::schema::room::{Joined, Opened};
    use meltos::user::UserId;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_tvn::branch::BranchName;
    use meltos_tvn::file_system::mock::MockFileSystem;
    use meltos_tvn::file_system::FileSystem;
    use meltos_tvn::io::bundle::BundleIo;
    use meltos_tvn::operation::init::Init;

    use crate::api::test_util::{http_call_with_deserialize, http_join, http_open_room, logged_in_app, open_room_request_with_options, ResponseConvertable};
    use crate::app;

    #[tokio::test]
    async fn failed_if_requested_join_not_exists_room() {
        let (_, mut app) = logged_in_app().await;
        let response = http_join(&mut app, &RoomId("invalid_id".to_string()), None).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn return_status_code_is_ok_if_joined_exists_room() {
        let session = MockUserSessionIo::default();
        let mut app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        mock.write("./some_text.txt", b"text file").unwrap();
        let room_id = http_open_room(&mut app, mock.clone()).await;

        let response = http_join(&mut app, &room_id, None).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn return_tvn_meta() {
        let session = MockUserSessionIo::default();
        let mut app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        mock.write("./workspace/some_text.txt", b"text file").unwrap();
        Init::new(BranchName::owner(), mock.clone()).execute().unwrap();
        let bundle = BundleIo::new(mock.clone()).create().unwrap();
        let open_request = open_room_request_with_options(Some(bundle), None);
        let room_id =  http_call_with_deserialize::<Opened>(&mut app, open_request)
            .await
            .room_id;
        let response = http_join(&mut app, &room_id, None).await;
        let meta = response.deserialize::<Joined>().await;
        assert_eq!(meta.bundle.branches.len(), 1);
    }

    #[tokio::test]
    async fn return_user_id() {
        let session = MockUserSessionIo::default();
        let mut app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        mock.write("./some_text.txt", b"text file").unwrap();
        let room_id = http_open_room(&mut app, mock.clone()).await;
        let response = http_join(&mut app, &room_id, Some(UserId::from("room"))).await;
        let meta = response.deserialize::<Joined>().await;
        assert_eq!(meta.user_id, UserId::from("room"));
    }
}
