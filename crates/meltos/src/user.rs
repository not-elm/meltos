use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Sha1)]
pub struct UserId(pub String);
impl_string_new_type!(UserId);

#[wasm_bindgen]
impl UserId {
    #[wasm_bindgen(constructor)]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn js_to_string(&self) -> String {
        self.0.clone()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display, Sha1)]
pub struct SessionId(pub String);

#[wasm_bindgen]
impl SessionId {
    #[wasm_bindgen(constructor)]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
}
