use axum::extract::Query;
use serde::Deserialize;

use meltos_core::channel::{ChannelMessage, MessageData};
use meltos_core::discussion::id::DiscussionId;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::session::user::SessionUser;

#[derive(Deserialize, Debug)]
pub struct Param {
    discussion_id: DiscussionId,
}


/// ディスカッションを削除します。
///
/// # Errors
///
/// ## StatusCode: 200(OK)
///
/// - [`Closed`](meltos_core::schema::discussion::global::Closed)
///
/// ## StatusCode: 500(INTERNAL_SERVER_ERROR)
///
/// -
///
#[tracing::instrument]
pub async fn close(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Query(param): Query<Param>,
) -> HttpResult {
    let closed = room
        .global_discussion(user_id.clone(), move |exe| exe.close(param.discussion_id))
        .await?;

    room.send_all_users(ChannelMessage {
        from: user_id,
        message: MessageData::DiscussionClosed(closed.clone()),
    })
        .await;

    Ok(closed.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use axum::http::{header, StatusCode};
    use tower::ServiceExt;

    use meltos_core::discussion::id::DiscussionId;
    use meltos_core::schema::error::{DiscussionNotExistsBody, ErrorResponseBodyBase};
    use meltos_core::schema::room::Opened;
    use meltos_tvc::file_system::memory::MemoryFileSystem;

    use crate::api::test_util::{http_create_discussion, http_discussion_close, http_open_room, mock_app, ResponseConvertable};

    #[tokio::test]
    async fn failed_if_not_exists_query() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs).await;
        http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, format!("session_id={session_id}"))
                    .uri(format!("/room/{room_id}/discussion/global/close"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn failed_no_exists_discussion() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, format!("session_id={session_id}"))
                    .uri(format!(
                        "/room/{room_id}/discussion/global/close?discussion_id=23232ada"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response
            .deserialize::<DiscussionNotExistsBody>()
            .await;
        assert_eq!(error, DiscussionNotExistsBody {
            base: ErrorResponseBodyBase {
                category: "discussion".to_string(),
                error_name: "DiscussionNotExists".to_string(),
                message: "discussion not exists; id: 23232ada".to_string(),
            },
            discussion_id: DiscussionId("23232ada".to_string()),
        });
    }

    #[tokio::test]
    async fn return_closed_command() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            session_id,
            room_id,
            ..
        } = http_open_room(&mut app, fs).await;
        let discussion_id =
            http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone())
                .await
                .meta
                .id;
        let closed =
            http_discussion_close(&mut app, &room_id, &session_id, discussion_id.clone()).await;
        assert_eq!(closed.discussion_id, discussion_id);
    }
}
