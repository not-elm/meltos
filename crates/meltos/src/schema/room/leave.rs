use crate::user::UserId;
use serde::{Deserialize, Serialize};

/// ユーザーがルームから退出されたことを表します。
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Left {
    /// 退出したユーザーのID
    pub user_id: UserId,
}
