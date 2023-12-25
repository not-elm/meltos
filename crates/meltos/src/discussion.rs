use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::MessageId;
use crate::user::UserId;

pub mod id;
pub mod message;


#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct DiscussionMeta {
    pub id: DiscussionId,
    pub creator: UserId,
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
        Self{
            meta,
            messages: Vec::new()
        }
    }
}


impl Discussion {
    pub fn new(creator: UserId) -> Self {
        Self {
            meta: DiscussionMeta {
                creator,
                id: DiscussionId::new(),
            },
            messages: Vec::new(),
        }
    }
}
