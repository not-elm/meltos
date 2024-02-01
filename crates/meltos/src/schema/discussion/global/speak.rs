use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageText};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

/// ディスカッションへのメッセージ送信リクエストを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Speak {
    /// メッセージを送信するディスカッションのID
    pub discussion_id: DiscussionId,

    /// メッセージ本文
    pub text: MessageText,
}

/// メッセージが新規に送信されたことを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Spoke {
    /// ディスカッションのID
    pub discussion_id: DiscussionId,

    /// 送信されたメッセージ情報
    pub message: Message,
}

#[wasm_bindgen]
impl Spoke {
    #[wasm_bindgen(constructor)]
    pub fn new(discussion_id: String, message: Message) -> Self {
        Self {
            discussion_id: DiscussionId(discussion_id),
            message,
        }
    }
}
