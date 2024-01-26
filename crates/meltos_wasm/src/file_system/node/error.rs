use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

pub type NodeFsResult<T = JsValue> = std::result::Result<T, Error>;


#[wasm_bindgen(module = "fs")]
extern "C" {
    #[derive(Debug)]
    pub type Error;

    #[wasm_bindgen(method, getter)]
    pub fn code(this: &Error) -> String;
}

impl Error {
    #[inline(always)]
    pub fn already_exists(&self) -> bool {
        self.code() == "EEXIST"
    }
}