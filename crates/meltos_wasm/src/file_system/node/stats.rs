use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[derive(Debug)]
    pub type Stats;

    #[wasm_bindgen(method, js_name = isFile)]
    pub fn is_file(this: &Stats) -> bool;

    #[wasm_bindgen(method, getter)]
    pub fn size(this: &Stats) -> usize;

    #[wasm_bindgen(method, getter, js_name = ctimeMs)]
    pub fn c_time_ms(this: &Stats) -> usize;

    #[wasm_bindgen(method, getter, js_name = mtimeMs)]
    pub fn m_time_ms(this: &Stats) -> usize;
}