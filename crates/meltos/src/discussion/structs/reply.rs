use crate::discussion::structs::id::DiscussionId;
use crate::discussion::structs::message::{MessageNo, MessageText};
use crate::user::UserId;
use serde::{Deserialize, Serialize};


#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct ReplyDiscussion {
    pub id: DiscussionId,
    pub messages: Vec<Reply>,
}


impl ReplyDiscussion {
    pub fn add_message(&mut self, user_id: UserId, message_text: MessageText) {
        self.messages.push(Reply {
            user_id,
            no: MessageNo(self.messages.len()),
            text: message_text,
        });
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Reply {
    pub user_id: UserId,
    pub no: MessageNo,
    pub text: MessageText,
}
