mod tvc;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    () => {
       $crate::wasm::log("\n")
    };
    ($($arg:tt)*) => {{
        $crate::wasm::log(&format!($($arg)*));
    }};
}
