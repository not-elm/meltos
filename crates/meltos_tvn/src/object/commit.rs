use serde::{Deserialize, Serialize};

use crate::error;
use crate::io::atomic::head::CommitText;
use crate::object::{AsObject, Object, ObjectHash};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Commit {
    pub parent: Option<ObjectHash>,
    pub text: CommitText,
    pub stage: ObjectHash,
}


impl AsObject for Commit {
    #[inline]
    fn as_obj(&self) -> error::Result<Object> {
        Ok(Object::compress(serde_json::to_vec(self)?)?)
    }
}


impl TryFrom<Object> for Commit {
    type Error = error::Error;

    #[inline]
    fn try_from(value: Object) -> Result<Self, Self::Error> {
        value.deserialize::<Commit>()
    }
}