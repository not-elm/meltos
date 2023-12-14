use serde::{Deserialize, Serialize};

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageId};
use crate::discussion::DiscussionMeta;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Created {
    pub meta: DiscussionMeta,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Spoke {
    pub discussion_id: DiscussionId,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Replied {
    pub reply_message_id: MessageId,
    pub reply: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Closed {
    pub discussion_id: DiscussionId,
}
