use wasm_bindgen::JsValue;

pub type Result<T = ()> = std::result::Result<T, JsValue>;

pub trait IntoJsResult<T> {
    fn into_js_result(self) -> Result<T>;
}


impl<T> IntoJsResult<T> for std::io::Result<T> {
    fn into_js_result(self) -> Result<T> {
        match self {
            Ok(out) => Ok(out),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}
