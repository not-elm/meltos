use axum::Json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::schema::room::{Kick, Kicked, Left};

use crate::api::{AsSuccessResponse, HttpResult};
use crate::error;
use crate::middleware::room::SessionRoom;
use crate::middleware::session::owner::SessionOwner;

/// RoomIdに対応するRoomに参加します。
///
///
/// # Errors
///
/// ## StatusCode: 200(OK)
///
/// - [`Kicked`](meltos::schema::room::Kicked) : 正常にユーザーがキックされた場合
///
/// ## StatusCode: 400(BAD_REQUEST)
///
/// - [`OwnerCannotKick`] : オーナー自身をキックしようとした場合
///
/// ## StatusCode: 401(UNAUTHORIZED)
///
/// - [`UserUnauthorized`](meltos::schema::error::ErrorResponseBodyBase) : 無効なセッションIDが指定された場合
///
/// ## StatusCode: 403(FORBIDDEN)
///
/// - [`PermissionDenied`] : ルームオーナー以外からのリクエストの場合
///
pub async fn kick(
    SessionRoom(room): SessionRoom,
    SessionOwner(owner): SessionOwner,
    Json(kick): Json<Kick>,
) -> HttpResult {
    if kick.users.contains(&room.owner) {
        return Err(error::Error::OwnerCannotKick.into());
    }

    for user_id in kick.users.iter() {
        if let Err(e) = room.leave(user_id.clone()).await {
            tracing::error!("{e}");
        }
    }

    room.send_all_users(ChannelMessage {
        from: owner,
        message: MessageData::Left(Left { users: kick.users.clone() }),
    })
        .await;

    Ok(Kicked { users: kick.users }.as_success_response())
}


#[cfg(test)]
mod tests {
    use tokio_tungstenite::tungstenite::http::StatusCode;

    use meltos::schema::room::{Joined, Kicked, Opened};
    use meltos::user::UserId;
    use meltos_tvc::file_system::memory::MemoryFileSystem;

    use crate::api::test_util::{fetch_request, http_call, http_join, http_kick, http_open_room, kick_request, mock_app, ResponseConvertable};

    #[tokio::test]
    async fn kicked_user1() {
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id: owner_session,
            ..
        } = http_open_room(&mut app, MemoryFileSystem::default()).await;

        let user1 = UserId::from("user1");
        let Joined {
            session_id: user_session,
            ..
        } = http_join(&mut app, &room_id, Some(user1.clone()))
            .await
            .deserialize()
            .await;

        let Kicked { users } = http_kick(&mut app, &room_id, &owner_session, vec![user1.clone()]).await;
        assert_eq!(users, vec![user1.clone()]);

        let response = http_call(&mut app, fetch_request(&room_id, &user_session)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }


    #[tokio::test]
    async fn failed_if_kick_owner_himself() {
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id,
            user_id,
            ..
        } = http_open_room(&mut app, MemoryFileSystem::default()).await;

        let response = http_call(&mut app, kick_request(&room_id, &session_id, vec![user_id.clone()])).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}