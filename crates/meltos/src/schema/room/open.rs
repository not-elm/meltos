use serde::{Deserialize, Serialize};

use crate::room::RoomId;
use meltos_tvn::io::bundle::Bundle;

use crate::user::{SessionId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Open {
    pub user_id: Option<UserId>,
    pub bundle: Bundle,
}

impl Open {
    #[inline]
    pub const fn new(bundle: Bundle, user_id: Option<UserId>) -> Self {
        Self {
            user_id,
            bundle,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opened {
    pub room_id: RoomId,
    pub user_id: UserId,
    pub session_id: SessionId,
}
