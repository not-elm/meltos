use crate::thread::message::{MessageNo, MessageText};
use crate::thread::ThreadId;
use serde::{Deserialize, Serialize};


#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct ReplyThread {
    pub id: ThreadId,
    pub messages: Vec<Reply>,
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Reply {
    pub no: MessageNo,
    pub text: MessageText,
}
