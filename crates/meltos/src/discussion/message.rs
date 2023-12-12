use std::ops::DerefMut;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Deref, Display};

use crate::discussion::id::DiscussionId;
use crate::user::UserId;

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Hash, Deref)]
pub struct Messages(Vec<Message>);


impl DerefMut for Messages {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub user_id: UserId,
    pub text: MessageText,
    pub reply_discussion: Option<DiscussionId>,
}


impl Message {
    #[inline(always)]
    pub fn new(user_id: UserId, text: MessageText) -> Message {
        Message {
            user_id,
            id: MessageId::default(),
            text,
            reply_discussion: None,
        }
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


#[repr(transparent)]
#[derive(
Eq,
PartialEq,
Clone,
Debug,
Hash,
Serialize,
Deserialize,
Display,
)]
pub struct MessageId(pub String);

impl Default for MessageId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}
