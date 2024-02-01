use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;

use meltos::user::UserId;

use crate::middleware::room::PathParam;
use crate::middleware::user::extract_session_id_from_cookie;
use crate::state::AppState;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct SessionOwner(pub UserId);

#[async_trait]
impl FromRequestParts<AppState> for SessionOwner {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session_id = extract_session_id_from_cookie(parts)?;
        let room_id = PathParam::new(parts, state).await?.room_id;
        let mut rooms = state.rooms.lock().await;
        let room = rooms.room(&room_id)?;
        let user_id = room.session.fetch(session_id).await?;
        if room.owner == user_id {
            Ok(Self(user_id))
        } else {
            Err(crate::error::Error::PermissionDenied.into())
        }
    }
}