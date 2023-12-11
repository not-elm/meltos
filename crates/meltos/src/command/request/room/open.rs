use serde::{Deserialize, Serialize};

use crate::user::UserToken;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Open {
    pub user_token: UserToken,
}


impl Open {
    #[inline]
    pub const fn new(user_token: UserToken) -> Self {
        Self { user_token }
    }
}
