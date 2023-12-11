use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use meltos_util::macros::Display;

use crate::discussion::structs::reply::{Reply, ReplyDiscussion};
use crate::user::UserId;

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Hash)]
pub struct Messages(Vec<Message>);


impl Deref for Messages {
    type Target = Vec<Message>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl DerefMut for Messages {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Message {
    pub user_id: UserId,
    pub no: MessageNo,
    pub text: MessageText,
    pub reply_discussion: ReplyDiscussion,
}


impl Message {
    #[inline(always)]
    pub fn new(user_id: UserId, no: MessageNo, text: MessageText) -> Message {
        Message {
            user_id,
            no,
            text,
            reply_discussion: ReplyDiscussion::default(),
        }
    }


    pub fn add_reply(&mut self, user_id: UserId, message_text: MessageText) -> Reply {
        self.reply_discussion.add_message(user_id, message_text)
    }
}


#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MessageText(String);
impl_string_new_type!(MessageText);


#[repr(transparent)]
#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Debug,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Display,
)]
pub struct MessageNo(pub usize);


#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum Want {
    No(MessageNo),
    LatestNo,
}
