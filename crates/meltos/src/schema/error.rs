use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorResponse {
    /// エラーの種別
    pub error_type: String,

    /// エラーのメッセージ
    pub message: String,
}