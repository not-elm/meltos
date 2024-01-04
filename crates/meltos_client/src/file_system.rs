use std::path::Path;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::js_sys::{Object, Uint8Array};

use meltos_tvc::file_system::FileSystem;

use crate::console_log;

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[derive(Debug)]
    type Stats;

    #[wasm_bindgen(method, js_name = isFile)]
    fn is_file(this: &Stats) -> bool;

    #[wasm_bindgen(js_name = readFileSync)]
    fn read_file_sync(path: &str) -> JsValue;

    #[wasm_bindgen(js_name = mkdirSync)]
    fn mkdir_sync(path: &str, options: &JsValue) -> Option<String>;

    #[wasm_bindgen(js_name = writeFileSync)]
    fn write_file_sync(path: &str, data: Vec<u8>, options: &JsValue);

    #[wasm_bindgen(js_name = readdirSync)]
    fn read_dir_sync(path: &str, options: JsValue) -> Vec<String>;

    #[wasm_bindgen(js_name = unlinkSync)]
    fn unlink_sync(path: &str);

    #[wasm_bindgen(js_name = existsSync)]
    fn exists_sync(path: &str) -> bool;

    #[wasm_bindgen(js_name = lstatSync)]
    fn lstat_sync(path: &str) -> Stats;
}

#[wasm_bindgen]
extern "C" {
    type Buffer;

    #[wasm_bindgen(method, getter)]
    fn buffer(this: &Buffer) -> Uint8Array;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    fn length(this: &Buffer) -> u32;
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct NodeFileSystem {
    pub workspace_folder: String,
}

#[wasm_bindgen]
impl NodeFileSystem {
    #[wasm_bindgen(constructor)]
    #[inline(always)]
    pub fn new(workspace_folder: String) -> Self {
        Self {
            workspace_folder,
        }
    }

    #[inline]
    pub fn path(&self, path: &str) -> String {
        if path == "./" || path == "." {
            self.workspace_folder.clone()
        } else if path.starts_with(&self.workspace_folder) {
            path.to_string()
        } else {
            format!("{}/{}", self.workspace_folder, path.replace("./", ""))
        }
    }
}

impl NodeFileSystem {
    fn read_files(&self, files: &mut Vec<String>, dir_path: &str) {
        for path in read_dir_sync(dir_path, JsValue::null()) {
            let path = format!("{dir_path}/{path}");
            console_log!("[entry] : {path}");
            if lstat_sync(&path).is_file() {
                console_log!("{path} is file");
                files.push(path.trim_start_matches(&self.workspace_folder).to_string());
            } else {
                console_log!("{path} is dir");
                self.read_files(files, &path);
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MkdirOptions {
    recursive: bool,
}

impl FileSystem for NodeFileSystem {
    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let path = self.path(path);
        console_log!("call write file path={path}");
        if let Some(dir) = Path::new(&path).parent() {
            let dir = dir.to_str().unwrap();
            if !exists_sync(dir) {
                let options = serde_wasm_bindgen::to_value(&MkdirOptions {
                    recursive: true,
                })
                .unwrap();
                mkdir_sync(dir, &options);
            }
        }
        write_file_sync(&path, buf.to_vec(), &Object::new());
        Ok(())
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let path = self.path(path);
        console_log!("call read file path={path}");
        if exists_sync(&path) {
            console_log!("read file is exists");
            let buffer = read_file_sync(&path);
            console_log!("read file = {buffer:?}");
            if buffer.is_string() {
                console_log!("file buffer = {:?}", &buffer.as_string());
                Ok(Some(buffer.as_string().unwrap().into_bytes()))
            } else {
                let buffer: Uint8Array = buffer.unchecked_into();
                let buffer = buffer.to_vec();
                console_log!("file buffer = {:?}", &buffer[0..30]);
                Ok(Some(buffer))
            }
        } else {
            console_log!("read file is not exists");
            Ok(None)
        }
    }

    #[inline]
    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let path = self.path(path);
        console_log!("call all_file_path! = {path}");
        let files = if exists_sync(&path) {
            let mut files = Vec::new();
            self.read_files(&mut files, &path);
            files
        } else {
            Vec::with_capacity(0)
        };
        console_log!("call all_file_path = {files:?}");
        Ok(files)
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        console_log!("call delete");
        let path = self.path(path);
        unlink_sync(&path);
        Ok(())
    }
}
