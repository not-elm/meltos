use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Deserialize, Serialize)]
pub struct ThreadId(pub String);


impl ThreadId {
    #[inline(always)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }


    #[inline]
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}


impl Default for ThreadId {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
