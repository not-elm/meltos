use crate::user::{SessionId, UserId};
use meltos_tvn::io::bundle::Bundle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Join {
    pub user_id: Option<UserId>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Joined {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub bundle: Bundle,
}
