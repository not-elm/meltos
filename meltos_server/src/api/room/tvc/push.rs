use axum::http::Response;
use axum::Json;
use meltos_tvc::io::bundle::Bundle;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

///
/// * save pushed data related to commits.
#[tracing::instrument]
pub async fn push(
    SessionRoom(room): SessionRoom,
    SessionUser(_): SessionUser,
    Json(bundle): Json<Bundle>,
) -> HttpResult {
    room.save_bundle(bundle)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::mock::MockFileSystem;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::operation;
    use meltos_tvc::operation::commit::Commit;
    use meltos_tvc::operation::stage;

    use crate::api::test_util::{http_open_room, owner_session_id, MockServerClient};
    use crate::app;

    #[tokio::test]
    async fn success() {
        let session = MockUserSessionIo::with_mock_users().await;
        let mock = MockFileSystem::default();
        let mut app = app(session, MockGlobalDiscussionIo::default());
        let room_id = http_open_room(&mut app, mock.clone()).await;
        mock.write("./workspace/src/hello.txt", b"hello").unwrap();
        stage::Stage::new(BranchName::owner(), mock.clone())
            .execute(".")
            .unwrap();
        Commit::new(BranchName::owner(), mock.clone())
            .execute("commit")
            .unwrap();
        let mut sender = MockServerClient::new(&mut app, room_id, owner_session_id());
        operation::push::Push::new(BranchName::owner(), mock.clone())
            .execute(&mut sender)
            .await
            .unwrap();
    }
}
