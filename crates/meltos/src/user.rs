use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Sha1)]
pub struct UserId(String);
impl_string_new_type!(UserId);

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display, Sha1)]
pub struct SessionId(pub String);
