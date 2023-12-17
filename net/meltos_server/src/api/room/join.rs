use axum::body::Body;
use axum::http::Response;
use serde::{Deserialize, Serialize};

use meltos::user::UserId;
use meltos_tvn::io::bundle::Bundle;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

/// RoomIdに対応するRoomに参加
/// Roomが存在しない場合はstatus code404が返される
///
/// レスポンスはRoomのメタデータ
///
pub async fn join(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
) -> HttpResult {
    let bundle = room.create_bundle()?;
    let room_meta = RoomMeta {
        user_id,
        bundle,
    };
    Ok(Response::builder()
        .body(Body::from(serde_json::to_string(&room_meta).unwrap().to_string()))
        .unwrap()
    )
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMeta {
    pub user_id: UserId,
    pub bundle: Bundle,
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use meltos::room::RoomId;
    use meltos::user::{SessionId, UserId};
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_backend::user::SessionIo;
    use meltos_tvn::file_system::FileSystem;
    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::room::join::RoomMeta;
    use crate::api::test_util::{
        http_join, http_open_room, logged_in_app,
        ResponseConvertable,
    };
    use crate::app;

    #[tokio::test]
    async fn failed_if_requested_join_not_exists_room() {
        let (session_id, mut app) = logged_in_app().await;
        let response = http_join(&mut app, &RoomId("invalid_id".to_string()), &session_id).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn return_status_code_is_ok_if_joined_exists_room() {
        let session = MockUserSessionIo::default();
        let owner_session = SessionId("owner".to_string());
        let user_session = SessionId("user".to_string());
        session
            .register(owner_session.clone(), UserId::from("owner"))
            .await
            .unwrap();
        session
            .register(user_session.clone(), UserId::from("user"))
            .await
            .unwrap();

        let mut app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        mock.write("./some_text.txt", b"text file").unwrap();
        let room_id = http_open_room(&mut app, mock.clone(), owner_session).await;

        let response = http_join(&mut app, &room_id, &user_session).await;
        assert_eq!(response.status(), StatusCode::OK);
    }


    #[tokio::test]
    async fn return_tvn_meta() {
        let session = MockUserSessionIo::default();
        let owner_session = SessionId("owner".to_string());
        let user_session = SessionId("user".to_string());
        session
            .register(owner_session.clone(), UserId::from("owner"))
            .await
            .unwrap();
        session
            .register(user_session.clone(), UserId::from("user"))
            .await
            .unwrap();

        let mut app = app(session, MockGlobalDiscussionIo::default());

        let mock = MockFileSystem::default();
        mock.write("./some_text.txt", b"text file").unwrap();
        let room_id = http_open_room(&mut app, mock.clone(), owner_session).await;

        let response = http_join(&mut app, &room_id, &user_session).await;
        let meta = response.deserialize::<RoomMeta>().await;
        assert_eq!(meta.bundle.branches.len(), 1);
    }


    #[tokio::test]
    async fn return_user_id() {
        let session = MockUserSessionIo::default();
        let owner_session = SessionId("owner".to_string());
        let user_session = SessionId("user".to_string());
        session
            .register(owner_session.clone(), UserId::from("owner"))
            .await
            .unwrap();
        session
            .register(user_session.clone(), UserId::from("user"))
            .await
            .unwrap();

        let mut app = app(session, MockGlobalDiscussionIo::default());
        let mock = MockFileSystem::default();
        mock.write("./some_text.txt", b"text file").unwrap();
        let room_id = http_open_room(&mut app, mock.clone(), owner_session).await;
        let response = http_join(&mut app, &room_id, &user_session).await;
        let meta = response.deserialize::<RoomMeta>().await;
        assert_eq!(meta.user_id, UserId::from("user"));
    }
}
