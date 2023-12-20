use crate::error;
use crate::object::{AsMeta, Decodable, Encodable, ObjMeta};

#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileObj(pub Vec<u8>);


impl AsMeta for FileObj {
    #[inline]
    fn as_meta(&self) -> crate::error::Result<ObjMeta> {
        Ok(ObjMeta::compress(self.encode()?)?)
    }
}


impl Encodable for FileObj {
    #[inline]
    fn encode(&self) -> error::Result<Vec<u8>> {
        let mut buf = b"FILE\0".to_vec();
        buf.extend(&self.0);
        Ok(buf)
    }
}


impl Decodable for FileObj {
    #[inline]
    fn decode(buf: &[u8]) -> error::Result<Self> {
        Ok(Self(buf[5..].to_vec()))
    }
}


#[cfg(test)]
mod tests {
    use crate::object::file::FileObj;
    use crate::object::{Decodable, Encodable};

    #[test]
    fn append_header_if_serialized() {
        let file = FileObj(b"hello".to_vec());
        let buf = file.encode().unwrap();
        assert_eq!(buf, b"FILE\0hello");
    }


    #[test]
    fn decode() {
        let file = FileObj(b"hello".to_vec());
        let buf = file.encode().unwrap();
        let file2 = FileObj::decode(&buf).unwrap();
        assert_eq!(file, file2);
    }
}
