use async_trait::async_trait;
use wasm_bindgen::{JsValue};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{ArrayBuffer, Uint8Array};

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
    pub async fn read_file_api(this: &VscodeNodeFs, path: &str) -> Option<JsValue>;

    #[wasm_bindgen(method, js_name = create_dir)]
    pub async fn create_dir_api(this: &VscodeNodeFs, path: &str) ;

    #[wasm_bindgen(method, js_name = read_dir)]
    pub async fn read_dir_api(this: &VscodeNodeFs, path: &str) -> Option<JsValue>;

    #[wasm_bindgen(method, js_name = delete)]
    pub async fn delete_api(this: &VscodeNodeFs, path: &str);
}


#[async_trait]
impl FileSystem for VscodeNodeFs {
    async fn stat(&self, _: &str) -> std::io::Result<Option<Stat>> {
        todo!()
    }

    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.write_file_api(path, buf.to_vec()).await;
        Ok(())
    }

    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.create_dir_api(path).await;
        Ok(())
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let Some(buf) = self.read_file_api(path).await else{
            return Ok(None);
        };
        Ok(Some(Uint8Array::new(&buf).to_vec()))
    }

    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let Some(entries) = self.read_dir_api(path).await else{
            return Ok(None);
        };
        Ok(ArrayBuffer::from(&entries).t)
    }

    async fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let files = self.all_files_in_api(path).await;
        todo!()
    }

    async fn delete(&self, path: &str) -> std::io::Result<()> {
        self.delete_api(path).await;
        Ok(())
    }
}

unsafe impl Send for VscodeNodeFs {}

unsafe impl Sync for VscodeNodeFs {}