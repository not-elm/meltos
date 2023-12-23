use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn fetch(SessionRoom(room): SessionRoom, SessionUser(_): SessionUser) -> HttpResult {
    let bundle = room.create_bundle()?;
    Ok(bundle.as_success_response())
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;

    use meltos_tvn::file_system::mock::MockFileSystem;

    use crate::api::test_util::{http_call, http_fetch, http_open_room, logged_in_app};

    #[tokio::test]
    async fn failed_if_not_logged_in() {
        let mock = MockFileSystem::default();
        let (_, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, mock.clone()).await;
        let response = http_call(
            &mut app,
            Request::builder()
                .uri(format!("/room/{room_id}/tvn/fetch"))
                .body(Body::empty())
                .unwrap(),
        )
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }


    #[tokio::test]
    async fn fetch() {
        let mock = MockFileSystem::default();
        let (session_id, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, mock.clone()).await;
        let _bundle = http_fetch(&mut app, &room_id, &session_id).await;
    }
}
