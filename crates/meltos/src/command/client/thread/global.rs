use serde::{Deserialize, Serialize};
use crate::user::UserId;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum GlobalThreadOrder {
    NewThreadNotify{
        creator: UserId
    },
}
