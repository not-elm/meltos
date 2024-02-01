use crate::user::UserId;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

/// ユーザーの強制退出を表します。
///
/// このリクエストはルームオーナーのみ受理されます。
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Kick {
    /// キックするユーザーの一覧
    pub users: Vec<UserId>,
}

/// ユーザーが強制退出されたことを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Kicked {
    /// キックされたユーザーの一覧
    pub users: Vec<UserId>,
}
