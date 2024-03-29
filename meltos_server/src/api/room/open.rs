use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::Json;
use axum::response::Response;

use meltos_core::room::RoomId;
use meltos_core::schema::room::Open;
use meltos_core::schema::room::Opened;
use meltos_core::user::{SessionId, UserId};
use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_util::serde::SerializeJson;

use crate::api::HttpResult;
use crate::api::room::response_error_exceed_bundle_size;
use crate::error;
use crate::room::{Room, Rooms};
use crate::state::config::AppConfigs;

/// 新規Roomを開きます。
///
/// # Errors
///
/// ## StatusCode: 413(PAYLOAD_TOO_LARGE)
///
/// - [`ExceedBundleSize`](crate::error::Error::ExceedBundleSize) : リクエスト時に送信されたバンドルのサイズが上限値を超えた場合
///
/// ## StatusCode: 500(INTERNAL_SERVER_ERROR)
///
/// - [`FailedCreatedRoom`](crate::error::Error::FailedCreatedRoom)  : サーバに十分な空き領域がない場合
///
#[tracing::instrument(skip(rooms), fields(configs, param), ret, level = "INFO")]
pub async fn open<Session, Discussion>(
    State(rooms): State<Rooms>,
    State(configs): State<AppConfigs>,
    Json(param): Json<Open>,
) -> HttpResult
    where
        Discussion: DiscussionIo + NewDiscussIo + 'static,
        Session: SessionIo + NewSessionIo + Debug + 'static,
{
    error_if_cannot_create_room()?;

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

    let capacity = param.get_capacity(configs.max_user_limits);
    let life_time = param.lifetime_duration(configs.room_limit_life_time_sec);
    // 現状Roomオーナーは`owner`固定
    let user_id = UserId::from("owner");

    let room = Room::open::<Discussion, Session>(user_id.clone(), capacity)?;
    let (user_id, session_id) = room.session.register(Some(user_id)).await?;
    let room_id = room.id.clone();

    if let Some(bundle) = param.bundle {
        room.save_bundle(bundle).await?;
    }

    rooms.insert_room(room, life_time).await;

    Ok(response_success_create_room(room_id, user_id, session_id, capacity))
}


/// ストレージの空き容量が5GIB以上ならばルームをたてられる
fn error_if_cannot_create_room() -> error::Result {
    let disk = sys_info::disk_info().unwrap();
    let free_gib = disk.free / 1024 / 1024;
    if 5 <= free_gib {
        Ok(())
    } else {
        Err(error::Error::FailedCreatedRoom)
    }
}

fn response_success_create_room(
    room_id: RoomId,
    user_id: UserId,
    session_id: SessionId,
    user_limits: u64,
) -> Response {
    Response::builder()
        .body(Body::from(
            Opened {
                room_id,
                user_id,
                session_id,
                capacity: user_limits,
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

    use meltos_core::schema::error::ErrorResponseBodyBase;
    use meltos_core::schema::room::Opened;
    use meltos_tvc::file_system::memory::MemoryFileSystem;
    use meltos_tvc::io::bundle::{Bundle, BundleObject};
    use meltos_tvc::object::{CompressedBuf, ObjHash};

    use crate::api::test_util::{
        create_discussion_request, http_call, mock_app, open_room_request,
        open_room_request_with_options, ResponseConvertable,
    };
    use crate::error;

    #[tokio::test]
    async fn it_return_room_id_and_session_id() -> error::Result {
        let app = mock_app();
        let fs = MemoryFileSystem::default();
        let response = app.oneshot(open_room_request(fs).await).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.room_id.0.len(), 40);
        assert_eq!(opened.session_id.0.len(), 40);
        Ok(())
    }

    #[tokio::test]
    async fn timeout() -> error::Result {
        let mut app = mock_app();
        let response = http_call(&mut app, open_room_request_with_options(None, Some(1), None)).await;
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
    async fn user_limits_is_1() -> error::Result {
        let app = mock_app();
        let response = app.oneshot(open_room_request_with_options(None, None, Some(1))).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.capacity, 1);

        Ok(())
    }


    /// サーバ側で設定された上限値を超えた場合、上限値がcapacityになる。
    ///
    /// テスト時の上限値は100
    #[tokio::test]
    async fn capacity_is_100_if_over() -> error::Result {
        let app = mock_app();
        let response = app.oneshot(open_room_request_with_options(None, None, Some(101))).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.capacity, 100);

        Ok(())
    }

    #[tokio::test]
    async fn capacity_is_1_if_specified_0() -> error::Result {
        let app = mock_app();
        let response = app.oneshot(open_room_request_with_options(None, None, Some(0))).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.capacity, 1);

        Ok(())
    }

    #[tokio::test]
    async fn success_if_bundle_less_than_100mb() {
        let app = mock_app();
        let request =
            open_room_request_with_options(Some(create_bundle_less_than_1024bytes()), None, None);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn failed_if_send_bundle_more_than_100mib() {
        let mut app = mock_app();
        let request =
            open_room_request_with_options(Some(create_bundle_more_than_1025bytes()), None, None);
        let response = http_call(&mut app, request).await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let error = response.deserialize::<ErrorResponseBodyBase>().await;
        assert_eq!(error, ErrorResponseBodyBase {
            category: "tvc".to_string(),
            error_name: "ExceedBundleSize".to_string(),
            message: "bundle size to exceed; actual_bundle_size: 1025, limit_bundle_size: 1024".to_string(),
        });
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
