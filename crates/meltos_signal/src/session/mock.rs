use async_trait::async_trait;
use std::collections::HashMap;

use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::session::{SessionId, SessionIo};
use crate::shared::SharedMutex;


#[derive(Default, Clone)]
pub struct MockSessionIo {
    map: SharedMutex<HashMap<SessionId, RTCSessionDescription>>,
}


#[async_trait]
impl SessionIo for MockSessionIo {
    async fn insert(
        &self,
        session_id: SessionId,
        session: RTCSessionDescription,
    ) -> crate::error::Result {
        self.map.lock().await.insert(session_id, session);
        Ok(())
    }

    async fn read(
        &self,
        session_id: SessionId,
    ) -> crate::error::Result<Option<RTCSessionDescription>> {
        Ok(self.map.lock().await.get(&session_id).cloned())
    }
}
