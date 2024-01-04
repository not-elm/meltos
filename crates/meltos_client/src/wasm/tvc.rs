use crate::file_system::NodeFileSystem;
use crate::tvc::TvcClient;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct WasmtvcClient(TvcClient<NodeFileSystem>);

#[wasm_bindgen]
impl WasmtvcClient {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new(branch_name: String, fs: NodeFileSystem) -> Self {
        Self(TvcClient::new(branch_name, fs))
    }
}
