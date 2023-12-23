use std::fmt::{Display, Formatter};
use std::io;

use auto_delegate::delegate;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::compression::gz::Gz;
use meltos_util::compression::CompressionBuf;
use meltos_util::macros::{Deref, Display};

use crate::encode::{Decodable, Encodable};
use crate::object::commit::CommitObj;
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
use crate::object::local_commits::LocalCommitsObj;
use crate::object::tree::TreeObj;
use crate::{error, impl_serialize_and_deserialize};

pub mod commit;
pub mod delete;
pub mod file;
pub mod local_commits;
pub mod tree;

#[delegate]
pub trait AsMeta {
    fn as_meta(&self) -> error::Result<ObjMeta>;
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
            compressed_buf: CompressedBuf(Gz.zip(&buf)?),
            buf,
        })
    }

    pub fn expand(compressed_buf: CompressedBuf) -> io::Result<Self> {
        let buf = Gz.unzip(&compressed_buf.0)?;
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
    Delete(DeleteObj),
    Tree(TreeObj),
    Commit(CommitObj),
    LocalCommits(LocalCommitsObj),
}

impl Obj {
    pub fn expand(buf: &CompressedBuf) -> error::Result<Self> {
        let buf = Gz.unzip(&buf.0)?;
        if buf.starts_with(FileObj::HEADER) {
            Ok(Obj::File(FileObj::decode(&buf)?))
        } else if buf.starts_with(DeleteObj::HEADER) {
            Ok(Obj::Delete(DeleteObj::decode(&buf)?))
        } else if buf.starts_with(TreeObj::HEADER) {
            Ok(Obj::Tree(TreeObj::decode(&buf)?))
        } else if buf.starts_with(CommitObj::HEADER) {
            Ok(Obj::Commit(CommitObj::decode(&buf)?))
        } else if buf.starts_with(LocalCommitsObj::HEADER) {
            Ok(Obj::LocalCommits(LocalCommitsObj::decode(&buf)?))
        } else {
            Err(crate::error::Error::InvalidObjBuffer(ObjHash::new(&buf)))
        }
    }

    pub fn file(self) -> error::Result<FileObj> {
        match self {
            Self::File(file) => Ok(file),
            _ => {
                Err(error::Error::InvalidObjType(
                    "File".to_string(),
                    self.to_string(),
                ))
            }
        }
    }

    pub fn commit(self) -> error::Result<CommitObj> {
        match self {
            Self::Commit(commit) => Ok(commit),
            _ => {
                Err(error::Error::InvalidObjType(
                    "commit".to_string(),
                    self.to_string(),
                ))
            }
        }
    }

    pub fn tree(self) -> error::Result<TreeObj> {
        match self {
            Self::Tree(tree) => Ok(tree),
            _ => {
                Err(error::Error::InvalidObjType(
                    "Tree".to_string(),
                    self.to_string(),
                ))
            }
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::File(_) => f.write_str("File"),
            Obj::Delete(_) => f.write_str("Delete"),
            Obj::Tree(_) => f.write_str("Tree"),
            Obj::Commit(_) => f.write_str("Commit"),
            Obj::LocalCommits(_) => f.write_str("LocalCommits"),
        }
    }
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

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Display)]
pub struct ObjHash(pub String);
impl_serialize_and_deserialize!(ObjHash);

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
#[derive(Debug, Eq, PartialEq, Clone, Hash, Ord, PartialOrd, Deref, Serialize, Deserialize)]
pub struct CompressedBuf(pub Vec<u8>);
