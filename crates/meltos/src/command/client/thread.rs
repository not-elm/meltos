use serde::{Deserialize, Serialize};

pub mod global;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(tag = "type", rename = "thread")]
pub enum ThreadOrder {
    Global(global::GlobalThreadOrder)
}
