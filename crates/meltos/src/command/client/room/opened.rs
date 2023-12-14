use serde::{Deserialize, Serialize};

use crate::room::RoomId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    pub room_id: RoomId,
}
