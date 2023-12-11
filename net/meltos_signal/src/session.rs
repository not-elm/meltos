use async_trait::async_trait;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use meltos::session::RoomId;

use crate::error;

pub mod mock;


#[async_trait]
#[auto_delegate::delegate]
pub trait SessionIo: Send + Sync {
    async fn insert(&self, session_id: RoomId, session: RTCSessionDescription) -> error::Result;


    async fn read(&self, session_id: RoomId) -> error::Result<Option<RTCSessionDescription>>;
}
