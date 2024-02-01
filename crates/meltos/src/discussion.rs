use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::Message;
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

