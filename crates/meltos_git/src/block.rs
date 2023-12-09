use crate::block::file::BlockHash;
use meltos_util::impl_string_new_type;
use serde::{Deserialize, Serialize};

mod dir;
mod file;


pub struct BlockTree;


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct BlockMeta {
    pub ty: BlockType,
    pub path: BlockPath,
    pub hash: BlockHash,
}


#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct BlockPath(String);
impl_string_new_type!(BlockPath);


#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    #[serde(rename = "file")]
    File,

    #[serde(rename = "dir")]
    Dir,
}
