use wasm_bindgen::prelude::wasm_bindgen;

pub mod file_system;
mod error;

pub mod tvc;

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



#[cfg(test)]
pub mod tests {
    use crate::file_system::node::NodeFileSystem;

    pub fn workspace_folder() -> String {
        "D://tmp".to_string()
    }


    pub fn node_fs() -> NodeFileSystem {
        NodeFileSystem::new(workspace_folder())
    }

}