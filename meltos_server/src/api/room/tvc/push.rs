use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::Json;
use serde_json::json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos_tvc::io::bundle::Bundle;

use crate::api::room::response_error_exceed_bundle_size;
use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;
use crate::state::config::AppConfigs;

///
/// * save pushed data related to commits.
#[tracing::instrument]
pub async fn push(
    State(configs): State<AppConfigs>,
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(bundle): Json<Bundle>,
) -> HttpResult {
    let bundle_data_size = bundle.obj_data_size();
    if configs.limit_bundle_size < bundle_data_size {
        return Err(response_error_exceed_bundle_size(
            bundle_data_size,
            configs.limit_bundle_size,
        ));
    }

    let repository_size = room.tvc_repository_size().await?;
    let actual_size = bundle_data_size + repository_size;
    if configs.limit_tvc_repository_size < actual_size {
        return Err(response_error_exceed_tvc_repository_size(
            configs.limit_tvc_repository_size,
            actual_size,
        ));
    }

    room.save_bundle(bundle.clone()).await?;
    room.send_all_users(ChannelMessage {
        from: user_id,
        message: MessageData::Pushed(bundle),
    })
    .await?;
    Ok(Response::default())
}

fn response_error_exceed_tvc_repository_size(
    limit_tvc_repository_size: usize,
    actual_size: usize,
) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(
            json!({
                "message" : "exceed tvc repository",
                "limit_tvc_repository_size": limit_tvc_repository_size,
                "actual_size" : actual_size
            })
            .to_string(),
        ))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::Response;
    use axum::Router;

    use meltos::room::RoomId;
    use meltos::schema::room::Opened;
    use meltos::user::SessionId;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::memory::MemoryFileSystem;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::operation;
    use meltos_tvc::operation::commit::Commit;
    use meltos_tvc::operation::stage;

    use crate::api::test_util::{http_open_room, mock_app, MockServerClient};

    #[tokio::test]
    async fn success_send_bundle() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs.clone()).await;
        fs.write_file("/workspace/src/hello.txt", b"hello").await.unwrap();
        let response = execute_tvc_operations(&mut app, &fs, room_id, session_id, branch).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn failed_if_exceed_bundle_size() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs.clone()).await;
        fs.write_sync("/workspace/src/hello.txt", &dummy_large_buf());
        let response = execute_tvc_operations(&mut app, &fs, room_id, session_id, branch).await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    /// 送信するbundleのデータサイズは上限を超えていないが、
    /// TVCのリポジトリサイズが上限値を超えてしまう場合、エラーになること
    ///
    /// Test中はTVCのリポジトリサイズが4048バイト
    #[tokio::test]
    async fn failed_if_reached_limit_tvc_repository_size() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let mut app = mock_app();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs.clone()).await;
        // push1
        fs.write_sync("/workspace/src/hello2.txt", &dummy_buf(1));
        let response = execute_tvc_operations(
            &mut app,
            &fs,
            room_id.clone(),
            session_id.clone(),
            branch.clone(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        // push2
        fs.write_sync("/workspace/src/hello3.txt", &dummy_buf(2));
        let response = execute_tvc_operations(
            &mut app,
            &fs,
            room_id.clone(),
            session_id.clone(),
            branch.clone(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        // push3
        fs.write_sync("/workspace/src/hello4.txt", &dummy_buf(3));
        let response = execute_tvc_operations(
            &mut app,
            &fs,
            room_id.clone(),
            session_id.clone(),
            branch.clone(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        // push3
        fs.write_sync("/workspace/src/hello5.txt", &dummy_buf(4));
        let response = execute_tvc_operations(&mut app, &fs, room_id, session_id, branch).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn execute_tvc_operations<Fs: FileSystem + Clone>(
        app: &mut Router,
        fs: &Fs,
        room_id: RoomId,
        session_id: SessionId,
        branch: BranchName,
    ) -> Response {
        stage::Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&branch, "commit").await.unwrap();
        let mut sender = MockServerClient::new(app, room_id, session_id);
        operation::push::Push::new(fs.clone())
            .execute(branch, &mut sender)
            .await
            .unwrap()
    }

    fn dummy_large_buf() -> Vec<u8> {
        // GZipで圧縮された際に1024bytesを超えるようにbuf作成
        vec![1; 1_000_000]
    }

    fn dummy_buf(v: u8) -> Vec<u8> {
        // GZipで圧縮された際になるべく1024bytesに近づくようにbuf作成
        vec![v; 700_000]
    }
}
