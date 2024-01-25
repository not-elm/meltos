use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::spawn_local;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{Array, Uint8Array};

use meltos_tvc::file_system::{FileSystem, Stat};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type VscodeNodeFs;

    #[wasm_bindgen(method)]
    pub async fn all_files_in_api(this: &VscodeNodeFs, path: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = write_file)]
    pub async fn write_file_api(this: &VscodeNodeFs, path: &str, buf: Vec<u8>);

    #[wasm_bindgen(method, js_name = read_file)]
    pub async fn read_file_api(this: &VscodeNodeFs, path: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = create_dir)]
    pub async fn create_dir_api(this: &VscodeNodeFs, path: &str);

    #[wasm_bindgen(method, js_name = read_dir)]
    pub async fn read_dir_api(this: &VscodeNodeFs, path: &str) -> JsValue;

    #[wasm_bindgen(method, js_name = delete)]
    pub async fn delete_api(this: &VscodeNodeFs, path: &str);
}

unsafe impl Send for VscodeNodeFs {}

unsafe impl Sync for VscodeNodeFs {}


#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct WasmFileSystem(Arc<Mutex<VscodeNodeFs>>);


impl WasmFileSystem {
    async fn lock(&self) -> MutexGuard<VscodeNodeFs> {
        self.0.lock().await
    }
}


impl From<VscodeNodeFs> for WasmFileSystem {
    fn from(value: VscodeNodeFs) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

#[async_trait]
impl FileSystem for WasmFileSystem {
    async fn stat(&self, _: &str) -> std::io::Result<Option<Stat>> {
        todo!()
    }

    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let fs = self.lock().await.clone();
        let buf = buf.to_vec();
        let path = path.to_string();
        spawn_local(async move {
            fs.write_file_api(&path, buf).await;
            Ok(())
        })
            .await
            .unwrap()
    }

    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        let fs = self.lock().await.clone();
        let path = path.to_string();
        spawn_local(async move {
            fs.create_dir_api(&path).await;
            Ok(())
        })
            .await
            .unwrap()
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let fs = self.lock().await.clone();
        let path = path.to_string();
        spawn_local(async move {
            let buf = fs.read_file_api(&path).await;
            if buf.is_undefined() {
                Ok(None)
            } else {
                Ok(Some(Uint8Array::new(&buf).to_vec()))
            }
        })
            .await
            .unwrap()
    }

    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let fs = self.lock().await.clone();
        let path = path.to_string();
        spawn_local(async move {
            let entries = fs.read_dir_api(&path).await;
            if entries.is_undefined() {
                Ok(None)
            } else {
                Ok(Some(Array::from(&entries).to_vec().into_iter().map(|v| v.as_string().unwrap()).collect()))
            }
        })
            .await
            .unwrap()
    }

    async fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let fs = self.lock().await.clone();
        let path = path.to_string();
        spawn_local(async move {
            let files = fs.all_files_in_api(&path).await;
            if files.is_undefined() {
                Ok(Vec::with_capacity(0))
            } else {
                Ok(Array::from(&files).to_vec().into_iter().map(|v| v.as_string().unwrap()).collect())
            }
        })
            .await
            .unwrap()
    }

    async fn delete(&self, path: &str) -> std::io::Result<()> {
        let fs = self.lock().await.clone();
        let path = path.to_string();
        spawn_local(async move {
            fs.delete_api(&path).await;
            Ok(())
        })
            .await
            .unwrap()
    }
}

