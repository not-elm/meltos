use wasm_bindgen::prelude::wasm_bindgen;

use crate::config::SessionConfigs;
use crate::error::JsResult;
use meltos_wasm::file_system::NodeFileSystem;
use crate::tvc::TvcClient;

#[wasm_bindgen]
pub struct WasmTvcClient(TvcClient<NodeFileSystem>);

#[wasm_bindgen]
impl WasmTvcClient {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new(branch_name: String, fs: NodeFileSystem) -> Self {
        Self(TvcClient::new(branch_name, fs))
    }


    #[inline]
    pub fn stage(&self, path: String) -> JsResult {
        self.0.stage(path)?;
        Ok(())
    }

    #[inline]
    pub fn commit(&self, text: String) -> JsResult {
        self.0.commit(text)?;
        Ok(())
    }


    #[inline]
    pub async fn push(&mut self, session_configs: SessionConfigs) -> JsResult {
        self.0.push(session_configs).await?;
        Ok(())
    }

    #[inline]
    pub fn merge(&self, source_branch: String) -> JsResult {
        let _ = self.0.merge(source_branch)?;
        Ok(())
    }


    #[inline]
    pub async fn fetch(&self, session_configs: SessionConfigs) -> JsResult {
        self.0.fetch(session_configs).await?;
        Ok(())
    }
}
