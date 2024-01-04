use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageId, MessageText};
use crate::discussion::DiscussionMeta;

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Create {
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Created {
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Speak {
    pub discussion_id: DiscussionId,
    pub text: MessageText,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Spoke {
    pub discussion_id: DiscussionId,
    pub text: Message,
}

#[wasm_bindgen]
impl Spoke {
    #[wasm_bindgen(constructor)]
    pub fn new(discussion_id: String, text: Message) -> Self {
        Self {
            discussion_id: DiscussionId(discussion_id),
            text,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Reply {
    pub to: MessageId,
    pub text: MessageText,
}

#[wasm_bindgen]
impl Reply {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(target_id: String, text: String) -> Self {
        Self {
            to: MessageId(target_id),
            text: MessageText(text),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Replied {
    pub to: MessageId,
    pub message: Message,
}

#[wasm_bindgen]
impl Replied {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(target_id: String, message: Message) -> Self {
        Self {
            to: MessageId(target_id),
            message,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Close {
    pub discussion_id: DiscussionId,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Closed {
    pub discussion_id: DiscussionId,
}
