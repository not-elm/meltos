use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;

use meltos::room::RoomId;
use meltos::schema::room::Open;
use meltos::schema::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_util::serde::SerializeJson;

use crate::api::room::response_error_exceed_bundle_size;
use crate::api::HttpResult;
use crate::room::{Room, Rooms};
use crate::state::config::AppConfigs;

#[tracing::instrument]
pub async fn open<Session, Discussion>(
    State(rooms): State<Rooms>,
    State(configs): State<AppConfigs>,
    Json(param): Json<Open>,
) -> HttpResult
where
    Discussion: DiscussionIo + NewDiscussIo + 'static,
    Session: SessionIo + NewSessionIo + Debug + 'static,
{
    let bundle_size = param
        .bundle
        .as_ref()
        .map(|b| b.obj_data_size())
        .unwrap_or_default();
    if configs.limit_bundle_size < bundle_size {
        return Err(response_error_exceed_bundle_size(
            bundle_size,
            configs.limit_bundle_size,
        ));
    }

    let life_time = param.lifetime_duration(configs.room_limit_life_time_sec);
    let user_id = param.user_id.unwrap_or_else(UserId::new);
    let room = Room::open::<Discussion, Session>(user_id.clone())?;
    let (user_id, session_id) = room.session.register(Some(user_id)).await?;
    let room_id = room.id.clone();

    if let Some(bundle) = param.bundle {
        room.save_bundle(bundle)?;
    }

    rooms.insert_room(room, life_time).await;

    Ok(response_success_create_room(room_id, user_id, session_id))
}

fn response_success_create_room(
    room_id: RoomId,
    user_id: UserId,
    session_id: SessionId,
) -> Response {
    Response::builder()
        .body(Body::from(
            Opened {
                room_id,
                user_id,
                session_id,
            }
            .as_json(),
        ))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::http::StatusCode;
    use tower::ServiceExt;

    use meltos::schema::room::Opened;
    use meltos_backend::discussion::global::mock::MockGlobalDiscussionIo;
    use meltos_backend::session::mock::MockSessionIo;
    use meltos_tvc::file_system::mock::MockFileSystem;
    use meltos_tvc::io::bundle::{Bundle, BundleObject};
    use meltos_tvc::object::{CompressedBuf, ObjHash};

    use crate::api::test_util::{
        create_discussion_request, http_call, mock_app, open_room_request,
        open_room_request_with_options, ResponseConvertable,
    };
    use crate::{app, error};

    #[tokio::test]
    async fn return_room_id_and_session_id() -> error::Result {
        let app = app::<MockSessionIo, MockGlobalDiscussionIo>();
        let fs = MockFileSystem::default();
        let response = app.oneshot(open_room_request(fs)).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.room_id.0.len(), 40);
        assert_eq!(opened.session_id.0.len(), 40);
        Ok(())
    }

    #[tokio::test]
    async fn timeout() -> error::Result {
        let mut app = app::<MockSessionIo, MockGlobalDiscussionIo>();
        let response = http_call(&mut app, open_room_request_with_options(None, Some(1))).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let opened = response.deserialize::<Opened>().await;
        let response = app
            .oneshot(create_discussion_request(
                "title".to_string(),
                opened.room_id,
                &opened.session_id,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    #[tokio::test]
    async fn success_if_bundle_less_than_100mb() {
        let app = mock_app();
        let request =
            open_room_request_with_options(Some(create_bundle_less_than_1024bytes()), None);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn failed_if_send_bundle_more_than_100mib() {
        let mut app = mock_app();
        let request =
            open_room_request_with_options(Some(create_bundle_more_than_1025bytes()), None);
        let response = http_call(&mut app, request).await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    fn create_bundle_less_than_1024bytes() -> Bundle {
        create_dummy_bundle(vec![1; 1024])
    }

    fn create_bundle_more_than_1025bytes() -> Bundle {
        create_dummy_bundle(vec![1; 1025])
    }

    fn create_dummy_bundle(mut buf: Vec<u8>) -> Bundle {
        buf.shrink_to_fit();
        Bundle {
            traces: Vec::with_capacity(0),
            objs: vec![BundleObject {
                hash: ObjHash::new(b"dummy"),
                compressed_buf: CompressedBuf(buf),
            }],
            branches: Vec::with_capacity(0),
        }
    }
}
