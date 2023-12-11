use serde::{Deserialize, Serialize};
use meltos_util::macros::Display;

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Deserialize, Serialize, Display)]
pub struct DiscussionId(pub String);


impl DiscussionId {
    #[inline(always)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}


impl Default for DiscussionId {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
