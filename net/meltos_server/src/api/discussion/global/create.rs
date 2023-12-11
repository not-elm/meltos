use std::fmt::Debug;

use axum::extract::State;
use axum::Json;

use meltos::command::request::discussion::global;
use meltos_backend::user::SessionIo;

use crate::api::{AsSuccessResponse, HttpResult};
use crate::room::Rooms;
use crate::state::SessionState;

#[tracing::instrument]
pub async fn create<Session: SessionIo + Debug>(
    State(session): State<SessionState<Session>>,
    State(rooms): State<Rooms>,
    Json(create): Json<global::Create>,
) -> HttpResult {
    let user_id = session.try_fetch_user_id(create.user_token).await?;
    let mut rooms = rooms.lock().await;
    let created = rooms
        .room_mut(&create.room_id)?
        .as_global_discussion_executor(user_id)
        .create()
        .await?;

    Ok(created.as_success_response())
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use tokio_tungstenite::tungstenite::handshake::client::Request;
    use tower::ServiceExt;

    use meltos::command::client::discussion::global::Created;
    use meltos::command::request::discussion::global::Create;
    use meltos_util::serde::SerializeJson;

    use crate::api::test_util::{logged_in_app, open_room};
    use crate::error;

    #[tokio::test]
    async fn return_created_command() -> error::Result {
        let (user_token, mut app) = logged_in_app().await;
        let room_id = open_room(&mut app, user_token.clone()).await;

        let request = Request::builder()
            .uri("/discussion/global/create")
            .method(http::method::Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(
                Create {
                    room_id,
                    user_token,
                }
                .as_json(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await?.to_bytes();
        serde_json::from_slice::<Created>(&bytes)?;

        Ok(())
    }
}
