use crate::discussion::id::DiscussionId;
use crate::discussion::message::{MessageNo, MessageText};
use crate::user::UserId;
use serde::{Deserialize, Serialize};


#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct ReplyDiscussion {
    pub id: DiscussionId,
    pub messages: Vec<ReplyMessage>,
}


impl ReplyDiscussion {
    pub fn add_message(&mut self, user_id: UserId, message_text: MessageText) -> ReplyMessage {
        let reply = ReplyMessage {
            user_id,
            no: MessageNo(self.messages.len()),
            message: message_text,
        };
        self.messages.push(reply.clone());
        reply
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct ReplyMessage {
    pub user_id: UserId,
    pub no: MessageNo,
    pub message: MessageText,
}
