use crate::structs::message::{MessageNo, MessageText};
use crate::structs::ThreadId;
use meltos_core::user::UserId;
use serde::{Deserialize, Serialize};


#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Default)]
pub struct ReplyThread {
    pub id: ThreadId,
    pub messages: Vec<Reply>,
}


impl ReplyThread {
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
