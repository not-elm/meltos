use wasm_bindgen::prelude::wasm_bindgen;
use crate::file_system::NodeFileSystem;
use crate::tvc::TvnClient;

#[wasm_bindgen]
pub struct WasmTvnClient(TvnClient<NodeFileSystem>);


#[wasm_bindgen]
impl WasmTvnClient {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new(branch_name: String, fs: NodeFileSystem) -> Self {
        Self(TvnClient::new(branch_name, fs))
    }
}