use axum::Json;

use meltos::channel::{ChannelMessage, MessageData};
use meltos::schema::discussion::global::Create;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn create(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(create): Json<Create>,
) -> HttpResult {
    let created = room
        .global_discussion(user_id.clone(), |exe| exe.create(create.title))
        .await?;

    room.send_all_users(ChannelMessage {
        from: user_id,
        message: MessageData::DiscussionCreated(created.clone()),
    })
    .await?;

    Ok(created.as_success_response())
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use meltos::schema::discussion::global::Created;
    use meltos::schema::room::Opened;
    use meltos_tvc::file_system::mock::MockFileSystem;

    use crate::api::test_util::{create_discussion_request, http_open_room, mock_app};
    use crate::error;

    #[tokio::test]
    async fn return_created_command() -> error::Result {
        let mut app = mock_app();
        let fs = MockFileSystem::default();
        let Opened {
            room_id,
            session_id,
            ..
        } = http_open_room(&mut app, fs).await;
        let request = create_discussion_request("title".to_string(), room_id, &session_id);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await?.to_bytes();
        serde_json::from_slice::<Created>(&bytes)?;

        Ok(())
    }
}
