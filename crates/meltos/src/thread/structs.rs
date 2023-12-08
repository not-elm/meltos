use crate::error;
use crate::thread::structs::id::ThreadId;
use crate::thread::structs::message::{Message, MessageNo, MessageText, Messages};
use crate::user::UserId;
use serde::{Deserialize, Serialize};


pub mod id;
pub mod message;
pub mod reply;


#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MessageThread {
    pub id: ThreadId,
    pub messages: Messages,
}

impl MessageThread {
    pub fn add_message(&mut self, user_id: UserId, message: MessageText) {
        let no = MessageNo(self.messages.len());
        self.messages.push(Message::new(user_id, no, message))
    }


    pub fn add_reply(
        &mut self,
        user_id: UserId,
        no: MessageNo,
        message: MessageText,
    ) -> error::Result {
        let target_message = self
            .messages
            .iter_mut()
            .find(|m| m.no == no)
            .ok_or(error::Error::MessageNoNotExists(no))?;
        target_message.add_reply(user_id, message);

        Ok(())
    }
}
