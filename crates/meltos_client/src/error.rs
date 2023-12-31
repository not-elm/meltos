use thiserror::Error;
use wasm_bindgen::JsValue;


pub type JsResult<T = ()> = std::result::Result<T, JsValue>;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest_wasm::Error),

    #[error(transparent)]
    Tvn(#[from] meltos_tvn::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}


impl From<crate::error::Error> for JsValue {
    #[inline(always)]
    fn from(value: Error) -> Self {
        JsValue::from_str(&value.to_string())
    }
}