use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Blob(Vec<u8>);

pub struct BlockHash(String);

pub struct BlockName(String);

pub struct BlockMeta {
    pub hash: BlockHash,
    pub name: BlockName,
}

pub struct Tree {}
