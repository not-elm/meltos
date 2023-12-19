use serde::{Deserialize, Serialize};

use crate::room::RoomId;
use crate::user::{SessionId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub session_id: SessionId
}
