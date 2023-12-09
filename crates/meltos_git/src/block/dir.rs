use crate::block::BlockMeta;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockDir(Vec<BlockMeta>);
