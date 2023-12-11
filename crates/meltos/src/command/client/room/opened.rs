use crate::room::RoomId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    pub room_id: RoomId,
}
