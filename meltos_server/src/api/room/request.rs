use axum::http::Response;
use axum::Json;

use meltos::channel::request::RequestMessage;

use crate::api::HttpResult;
use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;

#[tracing::instrument(ret, level = "INFO")]
pub async fn request(
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
    Json(request): Json<RequestMessage>,
) -> HttpResult {
    room.send_request(request).await?;
    Ok(Response::default())
}


