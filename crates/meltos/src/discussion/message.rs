use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

use crate::user::UserId;

#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub user_id: UserId,
    pub text: MessageText,
}

impl Message {
    #[inline(always)]
    pub fn new(user_id: UserId, text: MessageText) -> Message {
        Message {
            id: MessageId::new(),
            user_id,
            text,
        }
    }
}

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MessageText(String);
impl_string_new_type!(MessageText);

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize, Display, Sha1)]
pub struct MessageId(pub String);
