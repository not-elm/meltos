use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::discussion::id::DiscussionId;

/// ディスカッション削除のリクエストを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Close {
    /// 削除対象のディスカッションId
    pub discussion_id: DiscussionId,
}



/// ディスカッションが削除されたことを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Closed {
    /// 削除されたディスカッションのId
    pub discussion_id: DiscussionId,
}
