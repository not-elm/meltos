use serde::{Deserialize, Serialize};

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{MessageNo, MessageText};
use crate::room::RoomId;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Create {
    pub room_id: RoomId,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Speak {
    pub discussion_id: DiscussionId,
    pub message: MessageText,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Reply {
    pub discussion_id: DiscussionId,
    pub message_no: MessageNo,
    pub message: MessageText,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Close {
    pub discussion_id: DiscussionId,
}
