use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::session::user::SessionUser;

/// 現在のルームの状態を全て返します。
///
/// リクエスト側とルームの状態を同期するために使用されます。
pub async fn sync(SessionRoom(room): SessionRoom, SessionUser(_): SessionUser) -> HttpResult {
    let room_bundle = room.room_bundle().await?;
    Ok(room_bundle.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use meltos_core::discussion::{DiscussionBundle, MessageBundle};
    use meltos_core::discussion::message::MessageText;
    use meltos_core::schema::discussion::global::{Created, Speak, Spoke};
    use meltos_core::schema::room::Opened;
    use meltos_tvc::branch::BranchName;
    use meltos_tvc::file_system::FilePath;
    use meltos_tvc::file_system::memory::MemoryFileSystem;
    use meltos_tvc::io::bundle::BundleIo;
    use meltos_tvc::io::trace_tree::TraceTreeIo;
    use meltos_tvc::operation::commit::Commit;
    use meltos_tvc::operation::init::Init;
    use meltos_tvc::operation::push::Push;
    use meltos_tvc::operation::stage::Stage;

    use crate::api::test_util::{
        http_call, http_create_discussion, http_open_room, http_speak, http_sync, mock_app,
        MockServerClient, open_room_request_with_options, ResponseConvertable,
    };

    #[tokio::test]
    async fn it_return_discussion_states() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs.clone()).await;

        let Created {
            meta,
        } = http_create_discussion(&mut app, &session_id, "title".to_string(), room_id.clone())
            .await;

        let Spoke {
            message, ..
        } = http_speak(
            &mut app,
            &room_id,
            &session_id,
            Speak {
                discussion_id: meta.id.clone(),
                text: MessageText("speak".to_string()),
            },
        )
            .await;

        let room_bundle = http_sync(&mut app, &room_id, &session_id).await;
        assert_eq!(
            room_bundle.discussion,
            vec![DiscussionBundle {
                meta,
                messages: vec![MessageBundle {
                    message,
                    replies: Vec::with_capacity(0),
                }],
            }]
        )
    }

    #[tokio::test]
    async fn it_read_tvc_bundle() {
        let mut app = mock_app();
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let push = Push::new(fs.clone());
        let bundle = BundleIo::new(fs.clone());
        let traces = TraceTreeIo::new(fs.clone());

        init.execute(&branch).await.unwrap();
        let open_request = open_room_request_with_options(Some(bundle.create().await.unwrap()), None, None);
        let Opened {
            room_id,
            session_id,
            ..
        } = http_call(&mut app, open_request).await.deserialize().await;

        fs.write_sync("hello.txt", b"hello world!");
        stage.execute(&branch, ".").await.unwrap();
        commit.execute(&branch, "commit text").await.unwrap();
        let response = push
            .execute(
                branch.clone(),
                &mut MockServerClient::new(&mut app, room_id.clone(), session_id.clone()),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let mut room_bundle = http_sync(&mut app, &room_id, &session_id).await;
        assert_eq!(room_bundle.tvc.branches.len(), 1);
        assert_eq!(room_bundle.tvc.branches[0].branch_name, branch.clone());

        assert_eq!(room_bundle.tvc.traces.len(), 2);
        room_bundle.tvc.traces.sort();
        let hello_txt_hash = traces
            .read(&room_bundle.tvc.traces[1].commit_hash)
            .await
            .unwrap()
            .get(&FilePath::from_path("hello.txt"))
            .unwrap()
            .clone();

        assert!(room_bundle
            .tvc
            .objs
            .iter()
            .any(|o| o.hash == hello_txt_hash));
    }
}
