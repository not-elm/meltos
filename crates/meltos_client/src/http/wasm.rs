use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_core::schema::room::{Open, Opened};
use meltos_tvc::io::bundle::Bundle;

use crate::config::SessionConfigs;
use crate::error;

#[wasm_bindgen(module = "/js/http.js")]
extern {
    async fn open(body: String) -> JsValue;
}


#[derive(Debug, Clone)]
pub struct HttpClient {
    configs: SessionConfigs,
}


impl HttpClient {
    pub async fn open(
        _base_uri: &str,
        bundle: Option<Bundle>,
        lifetime_secs: Option<u64>,
        user_limits: Option<u64>,
    ) -> error::Result<Self> {
        let response = open(serde_json::to_string(&Open {
            bundle,
            lifetime_secs,
            user_limits,
        })
            .unwrap())
            .await
            .as_string()
            .unwrap();

        let opened: Opened = serde_json::from_str(&response).unwrap();
        Ok(Self {
            configs: SessionConfigs::from(opened)
        })
    }
}