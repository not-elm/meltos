use std::io;

use auto_delegate::delegate;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;
use meltos_util::macros::{Deref, Display};

use crate::error;
use crate::file_system::FilePath;
use crate::object::commit::CommitObj;
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
use crate::object::local_commits::LocalCommitsObj;
use crate::object::tree::TreeObj;

pub mod tree;
pub mod commit;
pub mod local_commits;
mod file;
mod delete;


#[delegate]
pub trait AsMeta {
    fn as_meta(&self) -> error::Result<ObjMeta>;
}


#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ObjMetaPath {
    pub file_path: FilePath,
    pub obj: ObjMeta,
}


impl ObjMetaPath {
    pub fn new(file_path: FilePath, buf: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            file_path,
            obj: ObjMeta::compress(buf)?,
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
            Self::LocalCommits(local_commits) => local_commits.as_meta()
        }
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
