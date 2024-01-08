use wasm_bindgen::prelude::wasm_bindgen;

use meltos_client::config::SessionConfigs;
use meltos_client::error::JsResult;
use meltos_client::tvc::TvcClient;

use crate::console_log;
use crate::file_system::vscode_node::VscodeNodeFs;

#[wasm_bindgen]
pub struct WasmTvcClient(TvcClient<VscodeNodeFs>);

#[wasm_bindgen]
impl WasmTvcClient {
    #[wasm_bindgen(constructor)]
    #[inline]
    pub fn new(branch_name: String, fs: VscodeNodeFs) -> Self {
        Self(TvcClient::new(branch_name, fs))
    }


    #[inline]
    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> JsResult<SessionConfigs> {
        let session_configs = self.0.open_room(lifetime_sec).await?;
        Ok(session_configs)
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


    #[inline]
    pub fn staging_files(&self) -> JsResult<Vec<String>> {
        let files = self.0.staging_files()?;
        Ok(files)
    }


    #[inline]
    pub fn exists_in_traces(&self, file_path: &str) -> JsResult<bool> {
        console_log!("INSPECT FILE = {file_path}");
        if let Some(traces) = self.0.traces()? {
            for (file, _) in traces.iter() {
                console_log!("FILE = {file}");
            }
        }


        let exists = self.0.exists_in_traces(file_path)?;
        Ok(exists)
    }

    #[inline]
    pub fn close(&self) -> JsResult {
        self.0.close()?;
        Ok(())
    }
}


