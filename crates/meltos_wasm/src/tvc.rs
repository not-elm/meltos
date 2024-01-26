use std::panic;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos::user::UserId;
use meltos_client::config::SessionConfigs;
use meltos_client::error::JsResult;
use meltos_client::tvc::TvcClient;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::FilePath;
use meltos_tvc::object::commit::CommitHash;
use meltos_tvc::object::ObjHash;

use crate::file_system::WasmFileSystem;
use crate::js_vec::{JsVecBranchCommitMeta, JsVecString};
use crate::vscode::FileChangeEventEmitter;

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmTvcClient {
    tvc: TvcClient<WasmFileSystem>,
    fs: WasmFileSystem,
}

#[wasm_bindgen]
impl WasmTvcClient {
    #[wasm_bindgen(constructor)]
    pub fn new(vscode_file_system: Option<FileChangeEventEmitter>) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let fs = WasmFileSystem::new(vscode_file_system);
        Self {
            tvc: TvcClient::new(fs.clone()),
            fs,
        }
    }

    #[inline(always)]
    pub fn fs(&self) -> WasmFileSystem {
        self.fs.clone()
    }

    #[inline]
    pub async fn init_repository(&self, branch_name: String) -> JsResult<CommitHash> {
        let commit_hash = self.tvc.init_repository(&BranchName(branch_name)).await?;
        Ok(commit_hash)
    }

    pub async fn unzip(&self, branch_name: String) -> JsResult {
        self.tvc.unzip(&BranchName(branch_name)).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn open_room(&mut self, lifetime_sec: Option<u64>, capacity: Option<u64>) -> JsResult<SessionConfigs> {
        let session_configs = self.tvc.open_room(lifetime_sec, capacity).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub async fn join_room(&mut self, room_id: String, user_id: Option<String>) -> JsResult<SessionConfigs> {
        let session_configs = self.tvc.join_room(room_id, user_id.map(UserId)).await?;
        Ok(session_configs)
    }

    #[inline(always)]
    pub async fn stage(&self, branch_name: String, path: String) -> JsResult {
        self.tvc.stage(&BranchName(branch_name), path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage(&self, file_path: &str) -> JsResult {
        self.tvc.un_stage(file_path).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn un_stage_all(&self) -> JsResult {
        self.tvc.un_stage_all().await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn commit(&self, branch_name: String, text: String) -> JsResult<CommitHash> {
        Ok(self.tvc.commit(&BranchName(branch_name), text).await?)
    }

    #[inline(always)]
    pub async fn push(&mut self, session_configs: &SessionConfigs) -> JsResult {
        self.tvc.push(session_configs.clone()).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn merge(&self, branch_name: String, source_commit_hash: String) -> JsResult {
        let _ = self.tvc.merge(BranchName(branch_name), CommitHash(ObjHash(source_commit_hash))).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn fetch(&self, session_configs: &SessionConfigs) -> JsResult {
        self.tvc.fetch(session_configs.clone()).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn staging_files(&self) -> JsResult<JsVecString> {
        let files = self.tvc.staging_files().await?;
        Ok(JsVecString(files))
    }

    #[inline(always)]
    pub async fn read_file_from_hash(&self, obj_hash: String) -> JsResult<Option<String>> {
        let content = self.tvc.read_file_from_hash(&ObjHash(obj_hash)).await?;
        Ok(content)
    }

    #[inline(always)]
    pub async fn all_branch_commit_metas(&self) -> JsResult<JsVecBranchCommitMeta> {
        let branches = self.tvc.all_branch_commit_metas().await?;
        Ok(JsVecBranchCommitMeta(branches))
    }

    #[inline(always)]
    pub async fn can_push(&self, branch_name: String) -> JsResult<bool> {
        Ok(self.tvc.can_push(&BranchName(branch_name)).await?)
    }


    #[inline(always)]
    pub async fn is_change(&self, branch_name: String, file_path: &str) -> JsResult<bool> {
        Ok(self.tvc.is_change(&BranchName(branch_name), &FilePath(file_path.to_string())).await?)
    }


    #[inline(always)]
    pub async fn find_obj_hash_from_traces(&self, branch_name: String, file_path: &str) -> JsResult<Option<ObjHash>> {
        let obj_hash = self.tvc.find_obj_hash_from_traces(&BranchName(branch_name), file_path).await?;
        Ok(obj_hash)
    }

    pub async fn sync_bundle(&self, bundle: &str) -> JsResult {
        self.tvc.save_bundle(serde_json::from_str(bundle).unwrap()).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn leave(&self, session_configs: &SessionConfigs) -> JsResult {
        self.tvc.leave(session_configs.clone()).await?;
        Ok(())
    }

    #[inline]
    pub async fn close(&self) -> JsResult {
        self.tvc.close().await?;
        Ok(())
    }
}


impl Default for WasmTvcClient {
    #[inline(always)]
    fn default() -> Self {
        Self::new(None)
    }
}