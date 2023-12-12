use axum::extract::Query;
use serde::Deserialize;

use meltos::discussion::id::DiscussionId;

use crate::api::HttpResult;
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
        .global_discussion(user_id, move |exe| exe.close(param.discussion_id))
        .await?;
    Ok(closed)
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http;
    use axum::http::{header, StatusCode};
    use tower::ServiceExt;

    use crate::api::test_util::{
        http_create_discussion, http_discussion_close, http_open_room, logged_in_app,
    };

    #[tokio::test]
    async fn failed_if_not_exists_query() {
        let (session_id, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, session_id).await;
        http_create_discussion(&mut app, room_id.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, "session_id=session_id")
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
        let (session_id, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, session_id).await;
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .header(header::SET_COOKIE, "session_id=session_id")
                    .uri(format!(
                        "/room/{room_id}/discussion/global/close?discussion_id=23232ada"
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
        let room_id = http_open_room(&mut app, session_id).await;
        let discussion_id = http_create_discussion(&mut app, room_id.clone())
            .await
            .meta
            .id;
        let closed = http_discussion_close(&mut app, &room_id, discussion_id.clone()).await;
        assert_eq!(closed.discussion_id, discussion_id);
    }
}
