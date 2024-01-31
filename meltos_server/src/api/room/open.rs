use std::fmt::Debug;

use axum::body::Body;
use axum::extract::State;
use axum::Json;
use axum::response::Response;

use meltos::room::RoomId;
use meltos::schema::room::Open;
use meltos::schema::room::Opened;
use meltos::user::{SessionId, UserId};
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_util::serde::SerializeJson;

use crate::api::HttpResult;
use crate::room::{Room, Rooms};
use crate::state::config::AppConfigs;

/// 新規Roomを開きます。
///
/// # Errors
///
/// ## StatusCode: 400(BAD_REQUEST)
///
/// - [`ExceedBundleSize`](crate::error::Error::ExceedBundleSize) : リクエスト時に送信されたバンドルのサイズが上限値を超えた場合
///
#[tracing::instrument(skip(rooms), fields(configs, param), ret, level = "INFO")]
pub async fn open<Session>(
    State(rooms): State<Rooms>,
    State(configs): State<AppConfigs>,
    Json(param): Json<Open>,
) -> HttpResult
    where
        Session: SessionIo + NewSessionIo + Debug + 'static,
{
    let capacity = param.get_capacity(configs.max_user_limits);
    let life_time = param.lifetime_duration(configs.room_limit_life_time_sec);
    // 現状Roomオーナーは`owner`固定
    let user_id = UserId::from("owner");

    let room = Room::open::<Session>(user_id.clone(), capacity)?;
    let (user_id, session_id) = room.session.register(Some(user_id)).await?;
    let room_id = room.id.clone();

    rooms.insert_room(room, life_time).await;

    Ok(response_success_create_room(room_id, user_id, session_id, capacity))
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

    use meltos::schema::error::ErrorResponseBodyBase;
    use meltos::schema::room::Opened;
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
        let response = http_call(&mut app, open_room_request_with_options(Some(1), None)).await;
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
        let response = app.oneshot(open_room_request_with_options(None, Some(1))).await.unwrap();
        let opened = response.deserialize::<Opened>().await;
        assert_eq!(opened.capacity, 1);

        Ok(())
    }
}
