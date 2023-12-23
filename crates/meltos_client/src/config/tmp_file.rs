use std::fs;

use async_trait::async_trait;

use crate::config::{SessionConfigs, SessionConfigsIo};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct TmpSessionConfigsIo;


#[async_trait]
impl SessionConfigsIo for TmpSessionConfigsIo {
    async fn save(&self, session_configs: SessionConfigs) -> crate::error::Result {
        fs::write(
            "./.room_configs",
            serde_json::to_string(&session_configs).unwrap(),
        )?;
        Ok(())
    }

    async fn load(&self) -> crate::error::Result<SessionConfigs> {
        let json = fs::read_to_string("./room-configs")?;
        Ok(serde_json::from_str(&json).unwrap())
    }
}
