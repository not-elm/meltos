use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{ArrayBuffer, Object};

use meltos_tvn::file_system::FileSystem;

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync)]
    fn read_file_sync(path: &str, options: &Object) -> Buffer;
}


#[wasm_bindgen]
extern "C" {
    type Buffer;

    #[wasm_bindgen(method, getter)]
    fn buffer(this: &Buffer) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    fn length(this: &Buffer) -> u32;
}


pub struct NodeFileSystem;


impl FileSystem for NodeFileSystem {
    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        todo!()
    }

    fn read(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        todo!()
    }

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        todo!()
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        todo!()
    }
}


