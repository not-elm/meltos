use serde::{Deserialize, Serialize};

use crate::error;
use crate::io::atomic::head::CommitText;
use crate::object::{AsMeta, ObjMeta, ObjHash};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CommitObj {
    pub parents: Vec<ObjHash>,
    pub text: CommitText,
    pub committed_objs_tree: ObjHash,
}


impl AsMeta for CommitObj {
    #[inline]
    fn as_meta(&self) -> error::Result<ObjMeta> {
        Ok(ObjMeta::compress(serde_json::to_vec(self)?)?)
    }
}


impl TryFrom<ObjMeta> for CommitObj {
    type Error = error::Error;

    #[inline]
    fn try_from(value: ObjMeta) -> Result<Self, Self::Error> {
        value.deserialize::<CommitObj>()
    }
}