use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Uint8Array;

#[wasm_bindgen]
extern "C" {
    pub type Buffer;

    #[wasm_bindgen(method, getter)]
    pub fn buffer(this: &Buffer) -> Uint8Array;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    pub fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &Buffer) -> u32;
}