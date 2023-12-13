use serde::{Deserialize, Serialize};

use crate::discussion::id::DiscussionId;
use crate::discussion::message::MessageId;
use crate::user::UserId;

pub mod id;
pub mod message;



#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct DiscussionMeta {
    pub id: DiscussionId,
    pub creator: UserId,
}


#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Discussion {
    pub meta: DiscussionMeta,
    pub messages: Vec<MessageId>,
}


impl Discussion {
    pub fn new(creator: UserId) -> Self {
        Self {
            meta: DiscussionMeta {
                creator,
                id: DiscussionId::new(),
            },
            messages: Vec::new(),
        }
    }
}
