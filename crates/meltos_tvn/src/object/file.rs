use crate::object::{AsMeta, ObjMeta};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileObj(pub Vec<u8>);


impl AsMeta for FileObj {
    #[inline]
    fn as_meta(&self) -> crate::error::Result<ObjMeta> {
        Ok(ObjMeta::compress(self.0.clone())?)
    }
}