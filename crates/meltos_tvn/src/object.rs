use std::io;

use auto_delegate::delegate;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;
use meltos_util::macros::{Deref, Display};

use crate::error;
use crate::object::commit::CommitObj;
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
use crate::object::local_commits::LocalCommitsObj;
use crate::object::tree::TreeObj;

pub mod commit;
pub mod delete;
pub mod file;
pub mod local_commits;
pub mod tree;


#[delegate]
pub trait AsMeta {
    fn as_meta(&self) -> error::Result<ObjMeta>;
}


#[delegate]
pub trait Encodable {
    fn encode(&self) -> error::Result<Vec<u8>>;
}


pub trait Decodable: Sized {
    fn decode(buf: &[u8]) -> error::Result<Self>;
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ObjMeta {
    pub hash: ObjHash,
    pub compressed_buf: CompressedBuf,
    pub buf: Vec<u8>,
}


impl ObjMeta {
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



#[derive(Debug, Clone)]
pub enum Obj {
    File(FileObj),
    Tree(TreeObj),
    Delete(DeleteObj),
    Commit(CommitObj),
    LocalCommits(LocalCommitsObj),
}


impl AsMeta for Obj {
    #[inline]
    fn as_meta(&self) -> error::Result<ObjMeta> {
        match self {
            Self::File(file) => file.as_meta(),
            Self::Tree(tree) => tree.as_meta(),
            Self::Delete(delete) => delete.as_meta(),
            Self::Commit(commit) => commit.as_meta(),
            Self::LocalCommits(local_commits) => local_commits.as_meta(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Display)]
pub struct ObjHash(pub String);


impl ObjHash {
    #[inline]
    pub fn new(buf: &[u8]) -> Self {
        Self(meltos_util::hash::hash(buf))
    }
}


impl Encodable for ObjHash {
    #[inline]
    fn encode(&self) -> error::Result<Vec<u8>> {
        Ok(self.0.as_bytes().to_vec())
    }
}


impl Decodable for ObjHash {
    #[inline]
    fn decode(buf: &[u8]) -> error::Result<Self> {
        let hash =
            String::from_utf8(buf.to_vec()).map_err(|_| error::Error::ObjHashBufferIsInvalid)?;
        Ok(Self(hash))
    }
}


#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd, Deref)]
pub struct CompressedBuf(pub Vec<u8>);
