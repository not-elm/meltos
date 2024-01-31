use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};

use meltos::room::RoomId;

use crate::room::Room;
use crate::state::AppState;

#[derive(Debug)]
pub struct SessionRoom(pub Room);

#[derive(TypedPath, Deserialize, Serialize)]
#[typed_path("/room/:room_id")]
pub struct PathParam {
    pub room_id: RoomId,
}

impl PathParam {
    pub async fn new(parts: &mut Parts, state: &AppState) -> Result<Self, Response> {
        let param = PathParam::from_request_parts(parts, state)
            .await
            .unwrap();
        Ok(param)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for SessionRoom {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let room_id = PathParam::new(parts, state).await?.room_id;
        let room = state.rooms.lock().await.room(&room_id)?;
        Ok(Self(room))
    }
}
