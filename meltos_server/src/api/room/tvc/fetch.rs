use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::session::user::SessionUser;

/// Room内のTvcリポジトリをバンドル化して取得します。
///
/// StatusCode: 200(OK)
///
/// - [`Bundle`](meltos_tvc::io::bundle::Bundle)
///
///
/// StatusCode: 500(INTERNAL_SERVER_ERROR)
///
/// - [`FailedTvcBody`](meltos_core::schema::error::FailedTvcBody): Tvc操作が失敗した場合
///
#[tracing::instrument]
pub async fn fetch(SessionRoom(room): SessionRoom, SessionUser(_): SessionUser) -> HttpResult {
    let bundle = room.create_bundle().await?;
    Ok(bundle.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;

    use meltos_core::schema::room::Opened;
    use meltos_tvc::file_system::memory::MemoryFileSystem;

    use crate::api::test_util::{http_call, http_fetch, http_open_room, mock_app};

    #[tokio::test]
    async fn failed_if_not_logged_in() {
        let fs = MemoryFileSystem::default();
        let mut app = mock_app();
        let Opened {
            room_id, ..
        } = http_open_room(&mut app, fs.clone()).await;
        let response = http_call(
            &mut app,
            Request::builder()
                .uri(format!("/room/{room_id}/tvc/fetch"))
                .body(Body::empty())
                .unwrap(),
        )
            .await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn fetch() {
        let fs = MemoryFileSystem::default();
        let mut app = mock_app();
        let opened = http_open_room(&mut app, fs.clone()).await;
        let _bundle = http_fetch(&mut app, &opened.room_id, &opened.session_id).await;
    }
}
