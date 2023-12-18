use async_trait::async_trait;
use reqwest::{header, Client};

use meltos::room::RoomId;
use meltos::user::SessionId;

use crate::room::tvn::TvnClient;

pub mod discussion;
pub mod tvn;

#[async_trait]
pub trait RoomClientIo<E = crate::error::Error>: Sized {
    /// Join the room.
    async fn join(session_id: SessionId, room_id: RoomId) -> std::result::Result<Self, E>;
}


pub struct RoomClient {
    client: Client,
    room_id: RoomId,
    session_id: SessionId,
    tvn: TvnClient,
}


#[async_trait]
impl RoomClientIo<crate::error::Error> for RoomClient {
    async fn join(session_id: SessionId, room_id: RoomId) -> Result<Self, crate::error::Error> {
        let client = Client::new();
        client
            .post(format!("http://localhost:3000/room/{room_id}/join"))
            .header(header::SET_COOKIE, format!("session_id={session_id}"))
            .send()
            .await?;

        Ok(Self {
            client,
            tvn: TvnClient::new(room_id.clone(), session_id.clone()),
            room_id,
            session_id,
        })
    }
}
