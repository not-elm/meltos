mod global;

use crate::order::thread::global::GlobalThreadCommand;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(tag = "type", rename = "thread")]
pub enum ThreadCommand {
    Global(GlobalThreadCommand),
}
