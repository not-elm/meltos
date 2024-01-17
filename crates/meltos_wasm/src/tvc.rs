use wasm_bindgen::prelude::wasm_bindgen;

use meltos_client::config::SessionConfigs;
use meltos_client::error::JsResult;
use meltos_client::tvc::{CommitMeta, TvcClient};
use meltos_tvc::object::commit::CommitHash;
use meltos_tvc::object::ObjHash;
use meltos_util::console_log;

use crate::file_system::vscode_node::VscodeNodeFs;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmTvcClient(TvcClient<VscodeNodeFs>);

#[wasm_bindgen]
impl WasmTvcClient {
    #[wasm_bindgen(constructor)]
    pub fn new(branch_name: &str, fs: VscodeNodeFs) -> Self {
        Self(TvcClient::new(branch_name.to_string(), fs))
    }


    #[inline]
    pub fn init_repository(&self) -> JsResult<CommitHash> {
        let commit_hash = self.0.init_repository()?;
        Ok(commit_hash)
    }

    #[inline]
    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> JsResult<SessionConfigs> {
        let session_configs = self.0.open_room(lifetime_sec).await?;
        Ok(session_configs)
    }

    #[inline]
    pub async fn join_room(
        &self,
        room_id: String,
        user_id: String,
    ) -> JsResult<SessionConfigs> {
        let session_configs = self.0.join_room(room_id, user_id).await?;
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
    pub async fn push(&mut self, session_configs: &SessionConfigs) -> JsResult {
        console_log!("PUSH!!!!!!!!!!!!!");
        self.0.push(session_configs.clone()).await?;
        Ok(())
    }

    #[inline]
    pub fn merge(&self, source_branch: String) -> JsResult {
        let _ = self.0.merge(source_branch)?;
        Ok(())
    }


    #[inline]
    pub async fn fetch(&self, session_configs: &SessionConfigs) -> JsResult {
        self.0.fetch(session_configs.clone()).await?;
        Ok(())
    }


    #[inline]
    pub fn staging_files(&self) -> JsResult<Vec<String>> {
        let files = self.0.staging_files()?;
        Ok(files)
    }


    #[inline]
    pub fn read_file_from_hash(&self, obj_hash: String) -> JsResult<Option<String>> {
        let content = self.0.read_file_from_hash(&ObjHash(obj_hash))?;
        Ok(content)
    }


    pub fn all_commit_metas(&self) -> JsResult<Vec<CommitMeta>> {
        let metas = self.0.all_commit_metas()?;
        Ok(metas)
    }


    pub fn exists_in_traces(&self, file_path: &str) -> JsResult<bool> {
        let exists = self.0.exists_in_traces(file_path)?;
        Ok(exists)
    }


    #[inline]
    pub fn close(&self) -> JsResult {
        self.0.close()?;
        Ok(())
    }
}


