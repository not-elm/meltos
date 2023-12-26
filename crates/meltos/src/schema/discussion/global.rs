use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::DiscussionMeta;
use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageId, MessageText};

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
    pub message: MessageText,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Spoke {
    pub discussion_id: DiscussionId,
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Reply {
    pub target_id: MessageId,
    pub text: MessageText,
}


#[wasm_bindgen]
impl Reply {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(target_id: String, text: String) -> Self {
        Self {
            target_id: MessageId(target_id),
            text: MessageText(text),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Replied {
    pub reply_message_id: MessageId,
    pub reply: Message,
}


#[wasm_bindgen]
impl Replied {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(target_id: String, reply: Message) -> Self {
        Self {
            reply_message_id: MessageId(target_id),
            reply,
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
