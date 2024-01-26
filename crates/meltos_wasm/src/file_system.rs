use async_trait::async_trait;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_tvc::file_system::{FileSystem, Stat};
use meltos_tvc::file_system::memory::MemoryFileSystem;
use meltos_util::console_log;

use crate::error;
use crate::error::IntoJsResult;
use crate::file_system::node::NodeFileSystem;
use crate::js_vec::{JsVecString, JsVecU8};
use crate::vscode::{CHANGE, CREATE, DELETE, FileChangeEventEmitter};

pub mod node;
pub mod vscode_node;


#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct WasmFileSystem {
    repository: NodeFileSystem,
    workspace: MemoryFileSystem,
    vscode: Option<FileChangeEventEmitter>,
}


#[wasm_bindgen]
impl WasmFileSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(vscode: Option<FileChangeEventEmitter>) -> Self {
        Self {
            vscode,
            ..Self::default()
        }
    }

    pub async fn stat_api(&self, path: &str) -> error::Result<Option<Stat>> {
        self.stat(path).await.into_js_result()
    }

    #[inline(always)]
    pub async fn write_file_api(&self, path: &str, buf: &[u8]) -> error::Result {
        self.write_file(path, buf).await.into_js_result()
    }

    #[inline(always)]
    pub async fn read_file_api(&self, path: &str) -> error::Result<Option<JsVecU8>> {
        Ok(self
            .read_file(path)
            .await
            .into_js_result()?
            .map(JsVecU8))
    }

    #[inline(always)]
    pub async fn create_dir_api(&self, path: &str) -> error::Result {
        self.create_dir(path).await.into_js_result()
    }

    #[inline(always)]
    pub async fn read_dir_api(&self, path: &str) -> error::Result<Option<JsVecString>> {
        Ok(self.read_dir(path).await.into_js_result()?.map(JsVecString))
    }


    #[inline(always)]
    pub async fn delete_api(&self, path: &str) -> error::Result {
        self.delete(path).await.into_js_result()
    }

    #[inline(always)]
    pub async fn exists_api(&self, path: &str) -> error::Result<bool> {
        Ok(self.stat_api(path).await?.is_some())
    }


    #[inline(always)]
    pub async fn all_files_in_api(&self, path: &str) -> error::Result<JsVecString> {
        Ok(JsVecString(self
            .all_files_in(path)
            .await
            .into_js_result()?))
    }
}


#[async_trait(? Send)]
impl FileSystem for WasmFileSystem {
    #[inline(always)]
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        self.fs(path).stat(path).await
    }

    #[inline(always)]
    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        console_log!("path = {path}");
        let exists = self.exists(path).await?;
        self.fs(path).write_file(path, buf).await?;
        self.notify(path, if exists { CHANGE } else { CREATE });
        Ok(())
    }

    #[inline(always)]
    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        let exists = self.exists(path).await?;
        self.fs(path).create_dir(path).await?;
        self.notify(path, if exists { CHANGE } else { CREATE });
        Ok(())
    }

    #[inline(always)]
    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        self.fs(path).read_file(path).await
    }

    #[inline(always)]
    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        self.fs(path).read_dir(path).await
    }

    #[inline(always)]
    async fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs(path).delete(path).await?;
        self.notify(path, DELETE);
        Ok(())
    }
}


impl Default for WasmFileSystem {
    #[inline(always)]
    fn default() -> Self {
        Self {
            repository: NodeFileSystem::new("D://tmp".to_string()),
            workspace: MemoryFileSystem::default(),
            vscode: None,
        }
    }
}


impl WasmFileSystem {
    fn fs(&self, path: &str) -> &dyn FileSystem {
        if path.starts_with(".meltos") ||
            path.starts_with("./.meltos") ||
            path.starts_with("/.meltos") {
            &self.repository
        } else {
            &self.workspace
        }
    }

    #[inline(always)]
    async fn exists(&self, uri: &str) -> std::io::Result<bool> {
        Ok(self.stat(uri).await?.is_some())
    }

    fn notify(&self, uri: &str, change_type: &str) {
        if let Some(vscode) = self.vscode.as_ref() {
            vscode.notify(uri, change_type);
        }
    }
}
