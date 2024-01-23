use thiserror::Error;
use wasm_bindgen::JsValue;

pub type JsResult<T = ()> = std::result::Result<T, JsValue>;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "wasm")]
    #[error(transparent)]
    Http(#[from] reqwest_wasm::Error),

    #[cfg(not(feature = "wasm"))]
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("please open or join room")]
    NotInitialized,

    #[error(transparent)]
    Tvc(#[from] meltos_tvc::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<crate::error::Error> for JsValue {
    #[inline(always)]
    fn from(value: Error) -> Self {
        JsValue::from_str(&value.to_string())
    }
}
