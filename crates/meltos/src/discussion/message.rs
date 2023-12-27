use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

use crate::user::UserId;


#[wasm_bindgen(getter_with_clone)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub user_id: UserId,
    pub text: MessageText,
}


#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(constructor)]
    #[inline(always)]
    pub fn new(user_id: String, text: String) -> Message {
        Message {
            id: MessageId::new(),
            user_id: UserId(user_id),
            text: MessageText(text),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MessageText(pub String);
impl_string_new_type!(MessageText);


#[wasm_bindgen]
impl MessageText {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(text: String) -> Self{
        Self(text)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize, Display, Sha1)]
pub struct MessageId(pub String);


#[wasm_bindgen]
impl MessageId {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(text: String) -> Self{
        Self(text)
    }
}