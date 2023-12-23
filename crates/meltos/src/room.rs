use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::macros::{Display, Sha1};

#[wasm_bindgen(getter_with_clone)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize, Display, Sha1)]
pub struct RoomId(pub String);
