use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::{Message, MessageId};
use crate::user::UserId;

pub mod id;
pub mod message;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct DiscussionMeta {
    pub id: DiscussionId,

    pub title: String,

    pub creator: UserId,
}

#[wasm_bindgen]
impl DiscussionMeta {
    #[wasm_bindgen(constructor)]
    pub fn new(id: DiscussionId, title: String, creator: UserId) -> Self {
        Self {
            id,
            title,
            creator,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiscussionBundle {
    pub meta: DiscussionMeta,
    pub messages: Vec<MessageBundle>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageBundle {
    pub message: Message,
    pub replies: Vec<Message>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Discussion {
    pub meta: DiscussionMeta,
    pub messages: Vec<MessageId>,
}

#[wasm_bindgen]
impl Discussion {
    #[wasm_bindgen(constructor)]
    pub fn from_meta(meta: DiscussionMeta) -> Self {
        Self {
            meta,
            messages: Vec::new(),
        }
    }
}

impl Discussion {
    pub fn new(title: String, creator: UserId) -> Self {
        Self {
            meta: DiscussionMeta {
                title,
                creator,
                id: DiscussionId::new(),
            },
            messages: Vec::new(),
        }
    }
}
