use crate::thread::message::{Message, MessageNo, MessageText};
use serde::{Deserialize, Serialize};

pub mod message;
pub mod reply;


#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Thread {
    pub id: ThreadId,
    pub line_no: usize,
    pub messages: Vec<Message>,
}


impl Thread {
    pub fn new(line_no: usize) -> Self {
        Self {
            id: ThreadId::new(),
            line_no,
            messages: Vec::new(),
        }
    }


    pub fn speak(&mut self, message_text: MessageText) {
        self.messages
            .push(Message::new(MessageNo(self.messages.len()), message_text));
    }
}


#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadId(pub String);


impl ThreadId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}
