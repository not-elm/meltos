use serde::{Deserialize, Serialize};

use meltos_util::macros::Sha1;

use crate::discussion::message::MessageText;
use crate::user::UserId;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ReplyMessage {
    pub id: ReplyId,
    pub user_id: UserId,
    pub text: MessageText,
}


impl ReplyMessage {
    #[inline]
    pub fn new(user_id: UserId, text: MessageText) -> Self {
        Self {
            id: ReplyId::new(),
            user_id,
            text,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Sha1, Clone, Hash)]
pub struct ReplyId(pub String);
