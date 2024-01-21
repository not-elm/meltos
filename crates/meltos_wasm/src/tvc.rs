use wasm_bindgen::prelude::wasm_bindgen;

use meltos_client::config::SessionConfigs;
use meltos_client::error::JsResult;
use meltos_client::tvc::{CommitMeta, TvcClient};
use meltos_tvc::io::bundle::Bundle;
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

    #[inline(always)]
    pub async fn open_room(&self, lifetime_sec: Option<u64>) -> JsResult<SessionConfigs> {
        let session_configs = self.0.open_room(lifetime_sec).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub async fn join_room(
        &self,
        room_id: String,
        user_id: String,
    ) -> JsResult<SessionConfigs> {
        let session_configs = self.0.join_room(room_id, user_id).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub fn stage(&self, path: String) -> JsResult {
        self.0.stage(path)?;
        Ok(())
    }

    #[inline(always)]
    pub fn un_stage(&self, file_path: &str) -> JsResult {
        self.0.un_stage(file_path)?;
        Ok(())
    }

    #[inline(always)]
    pub fn un_stage_all(&self) -> JsResult {
        self.0.un_stage_all()?;
        Ok(())
    }

    #[inline(always)]
    pub fn commit(&self, text: String) -> JsResult<CommitHash> {
        Ok(self.0.commit(text)?)
    }


    #[inline(always)]
    pub async fn push(&mut self, session_configs: &SessionConfigs) -> JsResult {
        console_log!("PUSH!!!!!!!!!!!!!");
        self.0.push(session_configs.clone()).await?;
        Ok(())
    }

    #[inline(always)]
    pub fn merge(&self, source_commit_hash: String) -> JsResult {
        let _ = self.0.merge(CommitHash(ObjHash(source_commit_hash)))?;
        Ok(())
    }


    #[inline(always)]
    pub async fn fetch(&self, session_configs: &SessionConfigs) -> JsResult {
        self.0.fetch(session_configs.clone()).await?;
        Ok(())
    }


    #[inline(always)]
    pub fn staging_files(&self) -> JsResult<Vec<String>> {
        let files = self.0.staging_files()?;
        Ok(files)
    }


    #[inline(always)]
    pub fn read_file_from_hash(&self, obj_hash: String) -> JsResult<Option<String>> {
        let content = self.0.read_file_from_hash(&ObjHash(obj_hash))?;
        Ok(content)
    }

    #[inline(always)]
    pub fn all_commit_metas(&self) -> JsResult<Vec<CommitMeta>> {
        let metas = self.0.all_commit_metas()?;
        Ok(metas)
    }

    #[inline(always)]
    pub fn find_obj_hash_from_traces(&self, file_path: &str) -> JsResult<Option<ObjHash>> {
        let obj_hash = self.0.find_obj_hash_from_traces(file_path)?;
        Ok(obj_hash)
    }

    #[inline(always)]
    pub fn save_bundle(&self, bundle: &Bundle) -> JsResult {
        self.0.save_bundle(bundle.clone())?;
        Ok(())
    }

    #[inline]
    pub fn close(&self) -> JsResult {
        self.0.close()?;
        Ok(())
    }
}


