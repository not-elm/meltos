use crate::structs::message::{Message, MessageNo, MessageText};
use crate::structs::ThreadId;

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct FileThread {
    pub id: ThreadId,
    pub line_no: usize,
    pub messages: Vec<Message>,
}


impl FileThread {
    pub fn new(line_no: usize) -> Self {
        Self {
            id: ThreadId::new(),
            line_no,
            messages: Vec::new(),
        }
    }

    // pub fn speak(&mut self, message_text: MessageText) {
    //     self.messages
    //         .push(Message::new(MessageNo(self.messages.len()), message_text));
    // }
}
