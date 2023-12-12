use crate::discussion::id::DiscussionId;
use crate::discussion::message::{MessageNo, MessageText};
use crate::user::UserId;
use serde::{Deserialize, Serialize};


#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct ReplyDiscussion {
    pub id: DiscussionId,
    pub messages: Vec<Reply>,
}


impl ReplyDiscussion {
    pub fn add_message(&mut self, user_id: UserId, message_text: MessageText) -> Reply {
        let reply = Reply {
            user_id,
            no: MessageNo(self.messages.len()),
            text: message_text,
        };
        self.messages.push(reply.clone());
        reply
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Reply {
    pub user_id: UserId,
    pub no: MessageNo,
    pub text: MessageText,
}
