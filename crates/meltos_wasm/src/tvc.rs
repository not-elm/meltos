use wasm_bindgen::prelude::wasm_bindgen;

use meltos::user::UserId;
use meltos_client::config::SessionConfigs;
use meltos_client::error::JsResult;
use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::FilePath;
use meltos_tvc::object::commit::CommitHash;
use meltos_tvc::object::ObjHash;

use crate::file_system::vscode_node::{MemoryFileSystem, WasmFileSystem};
use crate::js_vec::{JsVecBranchCommitMeta, JsVecString};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmTvcClient(TvcClient<WasmFileSystem>);

#[wasm_bindgen]
impl WasmTvcClient {
    #[wasm_bindgen(constructor)]
    pub fn new(fs: MemoryFileSystem, branch_name: Option<String>) -> Self {
        Self(TvcClient::new(fs.into(), branch_name.map(BranchName)))
    }

    #[inline]
    pub async fn init_repository(&self) -> JsResult<CommitHash> {
        let commit_hash = self.0.init_repository().await?;
        Ok(commit_hash)
    }

    pub async fn unzip(&self) -> JsResult{
        self.0.unzip().await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn open_room(&mut self, lifetime_sec: Option<u64>, capacity: Option<u64>) -> JsResult<SessionConfigs> {
        let session_configs = self.0.open_room(lifetime_sec, capacity).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub async fn join_room(&mut self, room_id: String, user_id: Option<String>) -> JsResult<SessionConfigs> {
        let session_configs = self.0.join_room(room_id, user_id.map(UserId)).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub async fn stage(&self, path: String) -> JsResult {
        self.0.stage(path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage(&self, file_path: &str) -> JsResult {
        self.0.un_stage(file_path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage_all(&self) -> JsResult {
        self.0.un_stage_all().await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn commit(&self, text: String) -> JsResult<CommitHash> {
        Ok(self.0.commit(text).await?)
    }

    #[inline(always)]
    pub async fn push(&mut self, session_configs: &SessionConfigs) -> JsResult {
        self.0.push(session_configs.clone()).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn merge(&self, source_commit_hash: String) -> JsResult {
        let _ = self.0.merge(CommitHash(ObjHash(source_commit_hash))).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn fetch(&self, session_configs: &SessionConfigs) -> JsResult {
        self.0.fetch(session_configs.clone()).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn staging_files(&self) -> JsResult<JsVecString> {
        let files = self.0.staging_files().await?;
        Ok(JsVecString(files))
    }

    #[inline(always)]
    pub async fn read_file_from_hash(&self, obj_hash: String) -> JsResult<Option<String>> {
        let content = self.0.read_file_from_hash(&ObjHash(obj_hash)).await?;
        Ok(content)
    }

    #[inline(always)]
    pub async fn all_branch_commit_metas(&self) -> JsResult<JsVecBranchCommitMeta> {
        let branches = self.0.all_branch_commit_metas().await?;
        Ok(JsVecBranchCommitMeta(branches))
    }

    #[inline(always)]
    pub async fn can_push(&self) -> JsResult<bool> {
        Ok(self.0.can_push().await?)
    }


    #[inline(always)]
    pub async fn is_change(&self, file_path: &str) -> JsResult<bool> {
        Ok(self.0.is_change(&FilePath(file_path.to_string())).await?)
    }


    #[inline(always)]
    pub async fn find_obj_hash_from_traces(&self, file_path: &str) -> JsResult<Option<ObjHash>> {
        let obj_hash = self.0.find_obj_hash_from_traces(file_path).await?;
        Ok(obj_hash)
    }

    pub async fn sync_bundle(&self, bundle: &str) -> JsResult {
        self.0.save_bundle(serde_json::from_str(bundle).unwrap()).await?;
        Ok(())
    }


    #[inline(always)]
    pub async fn leave(&self, session_configs: &SessionConfigs) -> JsResult {
        self.0.leave(session_configs.clone()).await?;
        Ok(())
    }

    #[inline]
    pub async fn close(&self) -> JsResult {
        self.0.close().await?;
        Ok(())
    }
}
