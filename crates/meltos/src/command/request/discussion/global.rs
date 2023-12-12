use serde::{Deserialize, Serialize};

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{MessageNo, MessageText};
use crate::room::RoomId;
use crate::user::UserId;

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
    discussion_id: DiscussionId,
    user_id: UserId,
    message_no: MessageNo,
    message: MessageText,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Close {
    discussion_id: DiscussionId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(tag = "type", rename = "global", rename_all = "snake_case")]
pub enum GlobalCmd {
    Create,

    Speak {
        discussion_id: DiscussionId,
        user_id: UserId,
        message: MessageText,
    },

    Reply {
        discussion_id: DiscussionId,
        user_id: UserId,
        message_no: MessageNo,
        message: MessageText,
    },

    Close {
        discussion_id: DiscussionId,
    },
}


#[cfg(test)]
mod tests {
    use crate::command::request::discussion::global::GlobalCmd;

    #[test]
    fn new_thread() {
        assert_eq!(
            serde_json::to_string(&GlobalCmd::Create).unwrap(),
            "{\"type\":\"create\"}"
        );
    }
}
