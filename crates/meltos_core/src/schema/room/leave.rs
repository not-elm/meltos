use serde::{Deserialize, Serialize};

use crate::user::UserId;

/// ユーザーがルームから退出されたことを表します。
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Left {
    /// 退出したユーザーのID
    pub users: Vec<UserId>,
}
