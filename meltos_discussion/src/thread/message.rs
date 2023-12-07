use crate::thread::reply::ReplyThread;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Message {
    pub no: MessageNo,
    pub text: MessageText,
    pub reply_thread: Option<ReplyThread>,
}


impl Message {
    #[inline(always)]
    pub const fn new(no: MessageNo, text: MessageText) -> Message {
        Message {
            no,
            text,
            reply_thread: None,
        }
    }
}


#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MessageText(String);


#[repr(transparent)]
#[derive(
    Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug, Hash, Default, Serialize, Deserialize,
)]
pub struct MessageNo(pub usize);
