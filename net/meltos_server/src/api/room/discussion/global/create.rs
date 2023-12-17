use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument]
pub async fn create(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
) -> HttpResult {
    let created = room.global_discussion(user_id, |exe| exe.create()).await?;

    Ok(created)
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use meltos::command::client::discussion::global::Created;

    use crate::api::test_util::{create_discussion_request, http_open_room, logged_in_app};
    use crate::error;

    #[tokio::test]
    async fn return_created_command() -> error::Result {
        let (user_token, mut app) = logged_in_app().await;
        let room_id = http_open_room(&mut app, user_token.clone()).await;
        let request = create_discussion_request(room_id);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await?.to_bytes();
        serde_json::from_slice::<Created>(&bytes)?;

        Ok(())
    }
}
