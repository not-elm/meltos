use wasm_bindgen::prelude::wasm_bindgen;

pub mod file_system;
mod error;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    () => {
       $crate::log("\n")
    };
    ($($arg:tt)*) => {{
        $crate::log(&format!($($arg)*));
    }};
}
