use serde::{Deserialize, Serialize};

use crate::discussion::structs::id::DiscussionId;
use crate::discussion::structs::message::{MessageNo, MessageText};
use crate::user::UserId;

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
