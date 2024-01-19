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
    use meltos::schema::room::Opened;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::file_system::mock::MockFileSystem;
    use meltos_tvc::operation;
    use meltos_tvc::operation::commit::Commit;
    use meltos_tvc::operation::stage;

    use crate::api::test_util::{http_open_room, mock_app, MockServerClient};

    #[tokio::test]
    async fn success_send_bundle() {
        let fs = MockFileSystem::default();
        let branch = BranchName::owner();
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs.clone()).await;
        fs.write_file("workspace/src/hello.txt", b"hello")
            .unwrap();
        stage::Stage::new(fs.clone())
            .execute(&branch, ".")
            .unwrap();
        Commit::new(fs.clone())
            .execute(&branch, "commit")
            .unwrap();
        let mut sender = MockServerClient::new(&mut app, room_id, session_id);
        operation::push::Push::new(fs.clone())
            .execute(branch, &mut sender)
            .await
            .unwrap();
    }
}
