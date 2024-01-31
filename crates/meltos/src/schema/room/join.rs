use serde::{Deserialize, Serialize};

use crate::user::{SessionId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Join {
    pub user_id: Option<UserId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Joined {
    pub user_id: UserId,
    pub session_id: SessionId,
}
