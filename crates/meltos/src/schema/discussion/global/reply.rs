use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageId, MessageText};

/// メッセージへの返信リクエストを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Reply {
    /// 返信先メッセージが存在するディスカッションID
    pub discussion_id: DiscussionId,

    /// 返信先のメッセージID
    pub to: MessageId,

    /// 返信メッセージ本文
    pub text: MessageText,
}

#[wasm_bindgen]
impl Reply {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(discussion_id: String, to: String, text: String) -> Self {
        Self {
            discussion_id: DiscussionId(discussion_id),
            to: MessageId(to),
            text: MessageText(text),
        }
    }
}


/// 返信メッセージが送信されたことを表します。
#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Replied {
    /// ディスカッションID
    pub discussion_id: DiscussionId,

    /// 返信先のメッセージID
    pub to: MessageId,

    /// 返信メッセージ本文
    pub message: Message,
}

#[wasm_bindgen]
impl Replied {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(discussion_id: String, target_id: String, message: Message) -> Self {
        Self {
            discussion_id: DiscussionId(discussion_id),
            to: MessageId(target_id),
            message,
        }
    }
}
