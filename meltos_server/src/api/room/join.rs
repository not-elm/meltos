use axum::Json;

use meltos::channel::{ResponseMessage, MessageData};
use meltos::schema::room::{Join, Joined};

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;

/// RoomIdに対応するRoomに参加します。
///
///
/// # Errors
/// ## StatusCode: 400(BAD_REQUEST)
///
/// - [`UserIdConflict`](meltos::schema::error::ErrorResponseBodyBase) : 既に同名のユーザーIDが存在していた場合
///
/// ## StatusCode: 401(UNAUTHORIZED)
///
/// - [`UserUnauthorized`](meltos::schema::error::ErrorResponseBodyBase) : 無効なセッションIDが指定された場合
///
/// ## StatusCode: 404(NOT_FOUND)
///
/// - [`RoomNotFound`](meltos::schema::error::ErrorResponseBodyBase) : Roomが存在しない場合
///
/// ## StatusCode: 429(TOO_MANY_REQUESTS)
///
/// - [`ReachedCapacity`](meltos::schema::error::ReachedCapacityBody) : ルームの定員に達した場合
///
pub async fn join(SessionRoom(room): SessionRoom, Json(join): Json<Join>) -> HttpResult {
    room.error_if_reached_capacity().await?;

    let (user_id, session_id) = room.session.register(join.user_id).await?;

    let joined = Joined {
        user_id: user_id.clone(),
        session_id
    };

    room.send_all_users(ResponseMessage {
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
    use meltos::schema::error::{ErrorResponseBodyBase, ReachedCapacityBody};
    use meltos::schema::room::{Joined, Opened};
    use meltos::user::UserId;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::session::mock::MockSessionIo;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::file_system::memory::MemoryFileSystem;
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
        let error = response.deserialize::<ErrorResponseBodyBase>().await;
        assert_eq!(
            error,
            ErrorResponseBodyBase {
                category: "session".to_string(),
                error_type: "RoomNotExists".to_string(),
                message: "room not exists".to_string(),
            }
        )
    }

    #[tokio::test]
    async fn status_code_is_ok_if_joined_exists_room() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        fs.write_file("./some_text.txt", b"text file")
            .await
            .unwrap();
        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, None).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn failed_if_reached_capacity() {
        let mut app = mock_app();

        let opened: Opened = http_call_with_deserialize(
            &mut app,
            open_room_request_with_options(
                None,
                None,
                Some(1), // room capacity
            ),
        )
            .await;

        let response = http_join(&mut app, &opened.room_id, None).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let error = response.deserialize::<ReachedCapacityBody>().await;
        assert_eq!(
            error,
            ReachedCapacityBody {
                base: ErrorResponseBodyBase {
                    category: "session".to_string(),
                    error_type: "ReachedCapacity".to_string(),
                    message: "reached capacity; capacity: 1".to_string(),
                },
                capacity: 1,
            }
        );
    }

    #[tokio::test]
    async fn it_return_user_id() {
        let mut app = app::<MockSessionIo, MockGlobalDiscussionIo>();
        let fs = MemoryFileSystem::default();
        fs.write_file("./some_text.txt", b"text file")
            .await
            .unwrap();
        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("tvc"))).await;
        let meta = response.deserialize::<Joined>().await;
        assert_eq!(meta.user_id, UserId::from("tvc"));
    }

    #[tokio::test]
    async fn failed_if_conflict_user_ids() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();

        let opened = http_open_room(&mut app, fs.clone()).await;
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("user1"))).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_join(&mut app, &opened.room_id, Some(UserId::from("user1"))).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response.deserialize::<ErrorResponseBodyBase>().await;
        assert_eq!(
            error,
            ErrorResponseBodyBase {
                category: "session".to_string(),
                error_type: "UserIdConflict".to_string(),
                message: "user id conflict; id: user1".to_string(),
            }
        );
    }

}
