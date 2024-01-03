use axum::extract::Query;
use serde::Deserialize;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::discussion::id::DiscussionId;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[derive(Deserialize, Debug)]
pub struct Param {
    discussion_id: DiscussionId,
}

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
    .await?;

    Ok(closed.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use axum::http::{header, StatusCode};
    use tower::ServiceExt;

    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::test_util::{
        http_create_discussion, http_discussion_close, http_open_room, logged_in_app,
    };

    #[tokio::test]
    async fn failed_if_not_exists_query() {
        let (session_id, mut app) = logged_in_app().await;
        let mock = MockFileSystem::default();
        let room_id = http_open_room(&mut app, mock).await;
        http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, format!("session_id={session_id}"))
                    .uri(format!("/tvc/{room_id}/discussion/global/close"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn failed_no_exists_discussion() {
        let (session_id, mut app) = logged_in_app().await;
        let mock = MockFileSystem::default();
        let room_id = http_open_room(&mut app, mock).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, format!("session_id={session_id}"))
                    .uri(format!(
                        "/tvc/{room_id}/discussion/global/close?discussion_id=23232ada"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn return_closed_command() {
        let (session_id, mut app) = logged_in_app().await;
        let mock = MockFileSystem::default();
        let room_id = http_open_room(&mut app, mock).await;
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
