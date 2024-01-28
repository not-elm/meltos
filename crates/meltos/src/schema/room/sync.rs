use crate::discussion::DiscussionBundle;
use meltos_tvc::io::bundle::Bundle;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomBundle {
    pub tvc: Bundle,
    pub discussion: Vec<DiscussionBundle>,
}
