use serde::{Deserialize, Serialize};
use meltos::user::UserId;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct UserRequest {
    pub from: UserId,

    pub id: String,

    pub data: String,
}
