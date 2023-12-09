use crate::block::BlockMeta;
use crate::stage::Stage;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;


#[derive(Eq, PartialEq, Copy, Clone, Deserialize, Serialize, Hash, Debug)]
pub struct Staged(bool);


#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Hash)]
pub struct StageBlockMeta {
    pub staged: Staged,
    pub meta: BlockMeta,
}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Mapping(Vec<StageBlockMeta>);
