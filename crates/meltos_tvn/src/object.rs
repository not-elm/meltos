use std::io;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;
use meltos_util::macros::{Deref, Display};

use crate::error;
use crate::file_system::FilePath;

pub mod tree;
pub mod commit;
pub mod local_commits;


pub trait AsObj {
    fn as_obj(&self) -> error::Result<Obj>;
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ObjMeta {
    pub file_path: FilePath,
    pub obj: Obj,
}

impl From<(FilePath, Obj)> for ObjMeta {
    #[inline(always)]
    fn from(value: (FilePath, Obj)) -> Self {
        Self {
            file_path: value.0,
            obj: value.1,
        }
    }
}

impl ObjMeta {
    pub fn new(file_path: FilePath, buf: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            file_path,
            obj: Obj::compress(buf)?,
        })
    }

    #[inline]
    pub const fn hash(&self) -> &ObjHash {
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
pub struct Obj {
    pub hash: ObjHash,
    pub compressed_buf: CompressedBuf,
    pub buf: Vec<u8>,
}

impl Obj {
    #[inline]
    pub fn deserialize<D: DeserializeOwned>(&self) -> error::Result<D> {
        Ok(serde_json::from_slice(&self.buf)?)
    }

    pub fn compress(buf: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            hash: ObjHash::new(&buf),
            compressed_buf: CompressedBuf(Gz.encode(&buf)?),
            buf,
        })
    }

    pub fn expand(compressed_buf: CompressedBuf) -> io::Result<Self> {
        let buf = Gz.decode(&compressed_buf.0)?;
        Ok(Self {
            hash: ObjHash::new(&buf),
            buf,
            compressed_buf,
        })
    }
}


impl AsRef<Obj> for Obj {
    #[inline]
    fn as_ref(&self) -> &Obj {
        self
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Display)]
pub struct ObjHash(pub String);

impl ObjHash {
    #[inline]
    pub fn serialize_to_buf(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    #[inline]
    pub fn from_serialized_buf(buf: &[u8]) -> error::Result<Self> {
        let hash = serde_json::from_slice::<ObjHash>(buf)?;
        if hash.0.is_empty() {
            Err(error::Error::ObjHashIsEmpty)
        } else {
            Ok(hash)
        }
    }

    #[inline]
    pub fn new(buf: &[u8]) -> Self {
        Self(meltos_util::hash::hash(buf))
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Deref)]
pub struct CompressedBuf(pub Vec<u8>);
