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

#[wasm_bindgen(getter_with_clone)]
impl Message {

    #[wasm_bindgen(constructor)]
    #[inline(always)]
    pub fn new(user_id: UserId, text: MessageText) -> Message {
        Message {
            id: MessageId::new(),
            user_id,
            text,
        }
    }
}


#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct MessageText(pub String);
impl_string_new_type!(MessageText);


#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize, Display, Sha1)]
pub struct MessageId(pub String);
