use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::file_system::node::error::NodeFsResult;
use crate::file_system::node::MkdirOptions;
use crate::file_system::node::stats::Stats;

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    pub fn read_file_sync(path: &str) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = mkdirSync, catch)]
    pub fn mkdir_sync(path: &str, options: MkdirOptions) -> NodeFsResult<Option<String>>;

    #[wasm_bindgen(js_name = writeFileSync)]
    pub fn write_file_sync(path: &str, data: Vec<u8>, options: &JsValue);

    #[wasm_bindgen(js_name = readdirSync, catch)]
    fn _read_dir_sync(path: &str, options: JsValue) -> NodeFsResult<Vec<String>>;

    #[wasm_bindgen(js_name = rmdirSync, catch)]
    fn _rm_dir_sync(path: &str) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = rmSync, catch)]
    fn _rm_sync(path: &str) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = existsSync, catch)]
    fn _exists_sync(path: &str) -> NodeFsResult<bool>;

    #[wasm_bindgen(js_name = lstatSync, catch)]
    fn _lstat_sync(path: &str) -> NodeFsResult<Stats>;
}


pub fn exists_sync(path: &str) -> std::io::Result<bool> {
    match _exists_sync(path) {
        Ok(exists) => Ok(exists),
        Err(e) => {
            Err(std::io::Error::other(format!(
                "failed fs.existsSysnc: {e:?}"
            )))
        }
    }
}

pub fn read_dir_sync(path: &str) -> std::io::Result<Option<Vec<String>>> {
    match _read_dir_sync(path, JsValue::null()) {
        Ok(entries) => Ok(Some(entries)),
        Err(e) => Err(std::io::Error::other(format!("failed read dir: {e:?}"))),
    }
}

#[inline(always)]
pub fn rm_sync(path: &str) -> std::io::Result<()> {
    _rm_sync(path).map_err(|e| std::io::Error::other(format!("failed fs.rmSync : {e:?}")))?;
    Ok(())
}

#[inline(always)]
pub fn rm_dir_sync(path: &str) -> std::io::Result<()> {
    _rm_dir_sync(path)
        .map_err(|e| std::io::Error::other(format!("failed fs.rmdirSync : {e:?}")))?;
    Ok(())
}

#[inline]
pub fn lstat_sync(path: &str) -> std::io::Result<Stats> {
    let stats = _lstat_sync(path)
        .map_err(|e| std::io::Error::other(format!("failed lstat_sync : {e:?}")))?;
    Ok(stats)
}

#[inline]
pub fn is_file(path: &str) -> std::io::Result<bool> {
    Ok(lstat_sync(path)?.is_file())
}
