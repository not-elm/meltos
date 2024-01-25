use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use meltos_client::tvc::BranchCommitMeta;

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsVecString(pub Vec<String>);







#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsVecBranchCommitMeta(pub Vec<BranchCommitMeta>);

