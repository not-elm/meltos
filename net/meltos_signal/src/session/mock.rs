use std::collections::HashMap;

use async_trait::async_trait;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use meltos_util::sync::arc_mutex::ArcMutex;

use crate::session::{RoomId, SessionIo};

#[derive(Default, Clone)]
pub struct MockSessionIo {
    map: ArcMutex<HashMap<RoomId, RTCSessionDescription>>,
}


#[async_trait]
impl SessionIo for MockSessionIo {
    async fn insert(
        &self,
        session_id: RoomId,
        session: RTCSessionDescription,
    ) -> crate::error::Result {
        self.map.lock().await.insert(session_id, session);
        Ok(())
    }

    async fn read(
        &self,
        session_id: RoomId,
    ) -> crate::error::Result<Option<RTCSessionDescription>> {
        Ok(self.map.lock().await.get(&session_id).cloned())
    }
}