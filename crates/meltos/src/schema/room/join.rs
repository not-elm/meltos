use crate::user::{SessionId, UserId};
use meltos_tvc::io::bundle::Bundle;
use serde::{Deserialize, Serialize};
use crate::discussion::DiscussionBundle;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Join {
    pub user_id: Option<UserId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Joined {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub bundle: Bundle,
    pub discussions: Vec<DiscussionBundle>
}
