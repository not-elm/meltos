use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageNo, MessageText, Messages};
use crate::discussion::reply::ReplyMessage;
use serde::{Deserialize, Serialize};

use crate::error;
use crate::user::UserId;

pub mod id;
pub mod message;
pub mod reply;


#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct DiscussionMeta {
    pub id: DiscussionId,
    pub creator: UserId,
}


#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Discussion {
    pub meta: DiscussionMeta,
    pub messages: Messages,
}


impl Discussion {
    pub fn new(creator: UserId) -> Self {
        Self {
            meta: DiscussionMeta {
                creator,
                id: DiscussionId::new(),
            },
            messages: Messages::default(),
        }
    }


    pub fn add_message(&mut self, user_id: UserId, message: MessageText) -> Message {
        let no = MessageNo(self.messages.len());
        let message = Message::new(user_id, no, message);
        self.messages.push(message.clone());
        message
    }


    pub fn add_reply(
        &mut self,
        user_id: UserId,
        no: MessageNo,
        message: MessageText,
    ) -> error::Result<ReplyMessage> {
        let target_message = self
            .messages
            .iter_mut()
            .find(|m| m.no == no)
            .ok_or(error::Error::MessageNoNotExists(no))?;
        Ok(target_message.add_reply(user_id, message))
    }
}
