// use async_trait::async_trait;
// use wasm_bindgen::JsValue;
// use wasm_bindgen::prelude::wasm_bindgen;
// use wasm_bindgen_futures::js_sys::{Array, Uint8Array};
// use meltos_tvc::file_system::{FileSystem, Stat};
// use meltos_util::console_log;
// 
// #[wasm_bindgen]
// extern "C" {
//     #[derive(Debug, Clone)]
//     pub type MemoryFileSystem;
// 
//     #[wasm_bindgen(method, js_name = all_files_in)]
//     pub async fn all_files_in_api(this: &MemoryFileSystem, path: &str) -> JsValue;
// 
//     #[wasm_bindgen(method, js_name = write_file)]
//     pub async fn write_file_api(this: &MemoryFileSystem, path: &str, buf: Vec<u8>);
// 
//     #[wasm_bindgen(method, js_name = read_file)]
//     pub async fn read_file_api(this: &MemoryFileSystem, path: &str) -> JsValue;
// 
//     #[wasm_bindgen(method, js_name = create_dir)]
//     pub async fn create_dir_api(this: &MemoryFileSystem, path: &str);
// 
//     #[wasm_bindgen(method, js_name = read_dir)]
//     pub async fn read_dir_api(this: &MemoryFileSystem, path: &str) -> JsValue;
// 
//     #[wasm_bindgen(method, js_name = delete)]
//     pub async fn delete_api(this: &MemoryFileSystem, path: &str);
// }
// 
// unsafe impl Send for MemoryFileSystem {}
// 
// unsafe impl Sync for MemoryFileSystem {}
// 
// #[derive(Debug, Clone)]
// pub struct WasmFileSystem(pub MemoryFileSystem);
// 
// impl From<MemoryFileSystem> for WasmFileSystem {
//     fn from(value: MemoryFileSystem) -> Self {
//         Self(value)
//     }
// }
// 
// 
// #[async_trait(? Send)]
// impl FileSystem for WasmFileSystem {
//     async fn stat(&self, _: &str) -> std::io::Result<Option<Stat>> {
//         todo!()
//     }
// 
//     async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
//         let buf = buf.to_vec();
//         let path = path.to_string();
//         self.0.write_file_api(&path, buf).await;
//         Ok(())
//     }
// 
//     async fn create_dir(&self, path: &str) -> std::io::Result<()> {
//         let path = path.to_string();
//         self.0.create_dir_api(&path).await;
//         Ok(())
//     }
// 
//     async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
//         let path = path.to_string();
//         let buf = self.0.read_file_api(&path).await;
// 
//         if buf.is_undefined() {
//             Ok(None)
//         } else {
//             Ok(Some(Uint8Array::new(&buf).to_vec()))
//         }
//     }
// 
//     async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
//         let path = path.to_string();
//         let entries = self.0.read_dir_api(&path).await;
//         if entries.is_undefined() {
//             Ok(None)
//         } else {
//             Ok(Some(Array::from(&entries).to_vec().into_iter().map(|v| v.as_string().unwrap()).collect()))
//         }
//     }
// 
//     async fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
//         let path = path.to_string();
//         let files = self.0.all_files_in_api(&path).await;
//         if files.is_undefined() {
//             Ok(Vec::with_capacity(0))
//         } else {
//             Ok(Array::from(&files).to_vec().into_iter().map(|v| v.as_string().unwrap()).collect())
//         }
//     }
// 
//     async fn delete(&self, path: &str) -> std::io::Result<()> {
//         let path = path.to_string();
//         self.0.delete_api(&path).await;
//         Ok(())
//     }
// }

