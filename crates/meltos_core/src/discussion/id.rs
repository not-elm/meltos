use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::macros::{Display, Sha1};

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Deserialize, Serialize, Display, Sha1)]
pub struct DiscussionId(pub String);

#[wasm_bindgen]
impl DiscussionId {
    #[wasm_bindgen(constructor)]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn js_to_string(&self) -> String {
        self.0.clone()
    }
}
