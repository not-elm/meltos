use serde::{Deserialize, Serialize};

use crate::object::{AsMeta, ObjMeta, ObjHash};



/// A struct that indicates that the object has been deleted.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct DeleteObj(pub ObjHash);


impl AsMeta for DeleteObj {
    #[inline]
    fn as_meta(&self) -> crate::error::Result<ObjMeta> {
        Ok(ObjMeta::compress(serde_json::to_vec(self)?)?)
    }
}