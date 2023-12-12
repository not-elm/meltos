use serde::{Deserialize, Serialize};

use crate::user::SessionId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Open {
    pub user_token: SessionId,
}


impl Open {
    #[inline]
    pub const fn new(user_token: SessionId) -> Self {
        Self { user_token }
    }
}
