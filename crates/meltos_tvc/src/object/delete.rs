use crate::error;

use crate::object::{AsMeta, Decodable, Encodable, ObjHash, ObjMeta};

/// A struct that indicates that the object has been deleted.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct DeleteObj(pub ObjHash);

impl DeleteObj {
    pub const HEADER: &'static [u8] = b"DELETE\0";
}

impl AsMeta for DeleteObj {
    #[inline]
    fn as_meta(&self) -> error::Result<ObjMeta> {
        Ok(ObjMeta::compress(self.encode()?)?)
    }
}

impl Encodable for DeleteObj {
    #[inline]
    fn encode(&self) -> error::Result<Vec<u8>> {
        let mut buf = b"DELETE\0".to_vec();
        buf.extend(&self.0.encode()?);
        Ok(buf)
    }
}

impl Decodable for DeleteObj {
    #[inline]
    fn decode(buf: &[u8]) -> error::Result<Self> {
        Ok(Self(ObjHash::decode(&buf[DeleteObj::HEADER.len()..])?))
    }
}

#[cfg(test)]
mod tests {
    use crate::object::delete::DeleteObj;
    use crate::object::{Decodable, Encodable, ObjHash};

    #[test]
    fn encode() {
        let delete_obj = DeleteObj(ObjHash::new(b"hello"));
        let buf = delete_obj.encode().unwrap();
        let mut expect = b"DELETE\0".to_vec();
        expect.extend(delete_obj.0.encode().unwrap());
        assert_eq!(buf, expect);
    }

    #[test]
    fn decode() {
        let delete_obj = DeleteObj(ObjHash::new(b"hello"));
        let buf = delete_obj.encode().unwrap();
        let decoded = DeleteObj::decode(&buf).unwrap();
        assert_eq!(delete_obj, decoded);
    }
}
