use serde::{Deserialize, Serialize};

use crate::error;
use crate::io::atomic::head::CommitText;
use crate::object::{AsObject, Obj, ObjHash};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CommitObj {
    pub parent: Option<ObjHash>,
    pub text: CommitText,
    pub stage: ObjHash,
}


impl AsObject for CommitObj {
    #[inline]
    fn as_obj(&self) -> error::Result<Obj> {
        Ok(Obj::compress(serde_json::to_vec(self)?)?)
    }
}


impl TryFrom<Obj> for CommitObj {
    type Error = error::Error;

    #[inline]
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        value.deserialize::<CommitObj>()
    }
}