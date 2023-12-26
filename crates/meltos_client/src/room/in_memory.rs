use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Uint8Array;

use meltos_tvn::file_system::FileSystem;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type MemFs;

    #[wasm_bindgen(constructor)]
    pub fn new() -> MemFs;

    #[wasm_bindgen(method, js_name = "readApi")]
    fn read_api(this: &MemFs, uri: String) -> Option<Uint8Array>;

    #[wasm_bindgen(method, js_name = "deleteApi")]
    fn delete_api(this: &MemFs, uri: String);

    #[wasm_bindgen(method, js_name = "writeApi")]
    fn write_api(this: &MemFs, uri: String, buf: Uint8Array);

    #[wasm_bindgen(method, js_name = "allPathApi")]
    fn all_path_api(this: &MemFs, path: String) -> Vec<String>;
}


impl FileSystem for MemFs {
    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.write_api(path.to_string(), Uint8Array::from(buf));
        Ok(())
    }


    #[inline]
    fn read(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        Ok(self.read_api(path.to_string()).map(|buf| buf.to_vec()))
    }

    #[inline]
    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        Ok(self.all_path_api(path.to_string()))
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.delete_api(path.to_string());
        Ok(())
    }
}