use crate::discussion::DiscussionMeta;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

/// ディスカッション作成のリクエストを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Create {
    /// 作成するディスカッションのタイトル
    pub title: String,
}

#[wasm_bindgen]
impl Create {
    #[wasm_bindgen(constructor)]
    pub fn new(title: String) -> Self {
        Self {
            title,
        }
    }
}

/// ディスカッションが作成されたことを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Created {
    /// 作成されたディスカッションのメタ情報
    pub meta: DiscussionMeta,
}

#[wasm_bindgen]
impl Created {
    #[wasm_bindgen(constructor)]
    pub fn new(meta: DiscussionMeta) -> Self {
        Self {
            meta,
        }
    }
}
