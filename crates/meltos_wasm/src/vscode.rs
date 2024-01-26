use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern {
    #[derive(Debug, Clone)]
    pub type FileChangeEventEmitter;

    #[wasm_bindgen(method)]
    pub fn notify(this: &FileChangeEventEmitter, uri: &str, change_type: &str);
}

pub const CREATE: &str = "create";
pub const CHANGE: &str = "change";
pub const DELETE: &str = "delete";
