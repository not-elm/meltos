use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos::room::RoomId;
use meltos::schema::room::Opened;
use meltos::user::{SessionId, UserId};

use crate::error;

mod node;
pub mod tmp_file;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionConfigs {
    pub room_id: RoomId,
    pub session_id: SessionId,
    pub user_id: UserId,
}

#[wasm_bindgen]
impl SessionConfigs {
    #[wasm_bindgen(constructor)]
    pub fn new(session_id: SessionId, room_id: RoomId, user_id: UserId) -> Self {
        Self {
            session_id,
            room_id,
            user_id,
        }
    }
}

impl From<Opened> for SessionConfigs {
    fn from(value: Opened) -> Self {
        Self {
            room_id: value.room_id,
            session_id: value.session_id,
            user_id: value.user_id,
        }
    }
}

#[async_trait]
pub trait SessionConfigsIo {
    async fn save(&self, session_configs: SessionConfigs) -> error::Result;

    async fn load(&self) -> error::Result<SessionConfigs>;
}
