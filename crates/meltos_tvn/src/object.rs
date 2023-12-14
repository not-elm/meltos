use std::io;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;
use meltos_util::macros::{Deref, Display};
use crate::error;
use crate::file_system::FilePath;

pub mod tree;
pub mod commit;


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ObjectMeta {
    pub file_path: FilePath,
    pub obj: Object,
}

impl From<(FilePath, Object)> for ObjectMeta {
    #[inline(always)]
    fn from(value: (FilePath, Object)) -> Self {
        Self {
            file_path: value.0,
            obj: value.1,
        }
    }
}

impl ObjectMeta {
    pub fn new(file_path: FilePath, buf: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            file_path,
            obj: Object::compress(buf)?,
        })
    }

    #[inline]
    pub const fn hash(&self) -> &ObjectHash {
        &self.obj.hash
    }

    #[inline]
    pub const fn compressed_buf(&self) -> &CompressedBuf {
        &self.obj.compressed_buf
    }

    #[inline]
    pub fn buf(&self) -> &[u8] {
        &self.obj.buf
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Object {
    pub hash: ObjectHash,
    pub compressed_buf: CompressedBuf,
    pub buf: Vec<u8>,
}

impl Object {
    #[inline]
    pub fn deserialize<D: DeserializeOwned>(&self) -> error::Result<D> {
        Ok(serde_json::from_slice(&self.buf)?)
    }

    pub fn compress(buf: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            hash: ObjectHash::new(&buf),
            compressed_buf: CompressedBuf(Gz.encode(&buf)?),
            buf,
        })
    }

    pub fn expand(compressed_buf: CompressedBuf) -> io::Result<Self> {
        let buf = Gz.decode(&compressed_buf)?;
        Ok(Self {
            hash: ObjectHash::new(&buf),
            buf,
            compressed_buf,
        })
    }
}


impl AsRef<Object> for Object {
    #[inline]
    fn as_ref(&self) -> &Object {
        self
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Display)]
pub struct ObjectHash(pub String);

impl ObjectHash {
    #[inline]
    pub fn serialize_to_buf(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    #[inline]
    pub fn from_serialized_buf(buf: &[u8]) -> error::Result<Self> {
        Ok(Self(serde_json::from_slice(buf)?))
    }

    #[inline]
    pub fn new(buf: &[u8]) -> Self {
        Self(meltos_util::hash::hash(buf))
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Deref)]
pub struct CompressedBuf(pub Vec<u8>);
