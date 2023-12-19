use serde::{Deserialize, Serialize};

use meltos_tvn::io::bundle::Bundle;

use crate::user::UserId;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Open {
    pub user_id: Option<UserId>,
    pub bundle: Bundle,
}


impl Open {
    #[inline]
    pub const fn new(bundle: Bundle, user_id: Option<UserId>) -> Self {
        Self {
            user_id,
            bundle,
        }
    }
}
