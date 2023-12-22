mod tmp_file;

use async_trait::async_trait;
use meltos::room::RoomId;
use meltos::user::{SessionId, UserId};
use serde::{Deserialize, Serialize};
use meltos::schema::response::room::Opened;
use crate::error;

#[derive(Debug, Clone, Serialize,Deserialize, Eq, PartialEq)]
pub struct SessionConfigs {
    pub room_id: RoomId,
    pub session_id: SessionId,
    pub user_id: UserId,
}


impl From<Opened> for SessionConfigs {
    fn from(value: Opened) -> Self {
        Self{
            room_id: value.room_id,
            session_id: value.session_id,
            user_id: value.user_id
        }
    }
}

#[async_trait]
pub trait SessionConfigsIo{
    async fn save(&self, session_configs: SessionConfigs) -> error::Result;

    async fn load(&self) -> error::Result<SessionConfigs>;
}