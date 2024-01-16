use serde::{Deserialize, Serialize};
use crate::user::UserId;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Left{
    pub user_id: UserId
}