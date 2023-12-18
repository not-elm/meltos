use axum::http::Response;
use axum::Json;

use meltos_tvn::operation::push::PushParam;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;


///
/// * save pushed data related to commits.
#[tracing::instrument]
pub async fn push(
    SessionRoom(room): SessionRoom,
    SessionUser(_): SessionUser,
    Json(param): Json<PushParam>,
) -> HttpResult {
    room.save_commits(param)?;
    Ok(Response::default())
}


#[cfg(test)]
mod tests {
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::user::mock::MockUserSessionIo;
    use meltos_tvn::branch::BranchName;
    use meltos_tvn::file_system::mock::MockFileSystem;
    use meltos_tvn::file_system::FileSystem;
    use meltos_tvn::operation;
    use meltos_tvn::operation::commit::Commit;
    use meltos_tvn::operation::stage;

    use crate::api::test_util::{http_open_room, owner_session_id, MockServerClient};
    use crate::app;

    #[tokio::test]
    async fn success() {
        let session = MockUserSessionIo::with_mock_users().await;
        let mock = MockFileSystem::default();
        let mut app = app(session, MockGlobalDiscussionIo::default());
        let room_id = http_open_room(&mut app, mock.clone(), owner_session_id()).await;
        mock.write("./src/hello.txt", b"hello").unwrap();
        stage::Stage::new(BranchName::main(), mock.clone())
            .execute(".")
            .unwrap();
        Commit::new(BranchName::main(), mock.clone())
            .execute("commit")
            .unwrap();
        let mut sender = MockServerClient::new(&mut app, room_id, owner_session_id());
        operation::push::Push::new(BranchName::main(), mock.clone())
            .execute(&mut sender)
            .await
            .unwrap();
    }
}
