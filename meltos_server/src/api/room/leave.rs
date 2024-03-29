use axum::extract::State;
use axum::http::Response;

use meltos_core::channel::{ChannelMessage, MessageData};
use meltos_core::schema::room::Left;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::session::user::SessionUser;
use crate::room::Rooms;

/// Roomから退出します。
///
/// - ルームクライアントからのリクエストの場合、ユーザー全員に[`Left`](meltos_core::schema::room::Left)が送信されます。
/// - ルームオーナーからのリクエストの場合、ルームは閉じられユーザー全員に[`ClosedRoom`]が送信されます。
///
/// ## StatusCode:
///
pub async fn leave(
    State(rooms): State<Rooms>,
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
) -> HttpResult {
    if room.owner == user_id {
        let mut rooms = rooms.lock().await;
        let room_id = room.id.clone();
        drop(room);
        rooms.delete(&room_id).await;
        Ok(Response::default())
    } else {
        room.leave(user_id.clone()).await?;
        let left = Left {
            users: vec![user_id.clone()],
        };

        room.send_all_users(ChannelMessage {
            from: user_id,
            message: MessageData::Left(left.clone()),
        })
            .await;
        Ok(left.as_success_response())
    }
}

#[cfg(test)]
mod tests {
    use axum::{http, Router};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{header, StatusCode};
    use axum::response::Response;

    use meltos_core::room::RoomId;
    use meltos_core::schema::room::{Joined, Opened};
    use meltos_core::user::{SessionId, UserId};
    use meltos_tvc::file_system::memory::MemoryFileSystem;

    use crate::api::test_util::{
        fetch_request, http_call, http_join, mock_app, open_room_request, ResponseConvertable,
    };
    use crate::error;

    #[tokio::test]
    async fn delete_room_if_owner_left() -> error::Result {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let response = http_call(&mut app, open_room_request(fs.clone()).await).await;
        let opened = response.deserialize::<Opened>().await;
        let response = http_leave(&mut app, &opened.room_id, &opened.session_id).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_leave(&mut app, &opened.room_id, &opened.session_id).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    #[tokio::test]
    async fn not_delete_room_if_user_left() -> error::Result {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let response = http_call(&mut app, open_room_request(fs.clone()).await).await;
        let opened = response.deserialize::<Opened>().await;
        let joined = http_join(&mut app, &opened.room_id, Some(UserId::from("session")))
            .await
            .deserialize::<Joined>()
            .await;

        let response = http_leave(&mut app, &opened.room_id, &joined.session_id).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response =
            http_call(&mut app, fetch_request(&opened.room_id, &joined.session_id)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response =
            http_call(&mut app, fetch_request(&opened.room_id, &opened.session_id)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_leave(&mut app, &opened.room_id, &opened.session_id).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_leave(&mut app, &opened.room_id, &opened.session_id).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }


    #[tokio::test]
    async fn deleted_joined_user_head() -> error::Result {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let response = http_call(&mut app, open_room_request(fs.clone()).await).await;
        let opened = response.deserialize::<Opened>().await;
        let joined = http_join(&mut app, &opened.room_id, Some(UserId::from("session")))
            .await
            .deserialize::<Joined>()
            .await;
        assert!(std::fs::read(format!("resources/{}/.meltos/refs/heads/{}", opened.room_id, joined.user_id)).is_ok());
        let response = http_leave(&mut app, &opened.room_id, &joined.session_id).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(std::fs::read(format!("resources/{}/.meltos/refs/heads/{}", opened.room_id, joined.user_id)).is_err());
        Ok(())
    }

    pub async fn http_leave(
        app: &mut Router,
        room_id: &RoomId,
        session_id: &SessionId,
    ) -> Response {
        http_call(app, leave_request(room_id, session_id)).await
    }

    pub fn leave_request(room_id: &RoomId, session_id: &SessionId) -> Request {
        Request::builder()
            .method(http::Method::DELETE)
            .uri(format!("/room/{room_id}"))
            .header(header::SET_COOKIE, format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap()
    }
}
