use axum::Json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::schema::room::{Join, Joined};

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;

/// RoomIdに対応するRoomに参加
/// Roomが存在しない場合はstatus code404が返される
///
/// レスポンスはRoomのメタデータ
///
pub async fn join(SessionRoom(room): SessionRoom, Json(join): Json<Join>) -> HttpResult {
    let (user_id, session_id) = room.session.register(join.user_id).await?;
    let bundle = room.create_bundle()?;
    let discussions = room.discussions().await?;
    let joined = Joined {
        user_id: user_id.clone(),
        session_id,
        bundle,
        discussions,
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
    use meltos::schema::error::ErrorResponseBodyBase;
    use meltos::schema::room::{Joined, Opened};
    use meltos::user::UserId;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::session::mock::MockSessionIo;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::file_system::mock::MockFileSystem;
    use meltos_tvc::io::bundle::BundleIo;
    use meltos_tvc::operation::init::Init;

    use crate::api::test_util::{
        http_call_with_deserialize, http_join, http_open_room, mock_app,
        open_room_request_with_options, ResponseConvertable,
    };
    use crate::app;

    #[tokio::test]
    async fn failed_if_requested_join_not_exists_room() {
        let mut app = mock_app();
        let response = http_join(&mut app, &RoomId("invalid_id".to_string()), None).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn status_code_is_ok_if_joined_exists_room() {
        let mut app = mock_app();
        let fs = MockFileSystem::default();
        fs.write_file("./some_text.txt", b"text file").unwrap();
        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, None).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_return_tvc_meta() {
        let mut app = app::<MockSessionIo, MockGlobalDiscussionIo>();
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        fs.force_write("workspace/some_text.txt", b"text file");
        Init::new(fs.clone()).execute(&branch).unwrap();
        let bundle = BundleIo::new(fs.clone()).create().unwrap();
        let open_request = open_room_request_with_options(Some(bundle), None, None);
        let room_id = http_call_with_deserialize::<Opened>(&mut app, open_request)
            .await
            .room_id;
        let response = http_join(&mut app, &room_id, None).await;
        let meta = response.deserialize::<Joined>().await;
        assert_eq!(meta.bundle.branches.len(), 1);
    }

    #[tokio::test]
    async fn it_return_user_id() {
        let mut app = app::<MockSessionIo, MockGlobalDiscussionIo>();
        let fs = MockFileSystem::default();
        fs.write_file("./some_text.txt", b"text file").unwrap();
        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("tvc"))).await;
        let meta = response.deserialize::<Joined>().await;
        assert_eq!(meta.user_id, UserId::from("tvc"));
    }

    #[tokio::test]
    async fn failed_if_conflict_user_ids() {
        let mut app = mock_app();
        let fs = MockFileSystem::default();

        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("user1"))).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("user1"))).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response.deserialize::<ErrorResponseBodyBase>().await;
        assert_eq!(error, ErrorResponseBodyBase {
            error_type: "session".to_string(),
            message: "user id conflict; id: user1".to_string()
        });
    }
}
