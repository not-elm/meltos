use crate::config::{SessionConfigs, SessionConfigsIo};
use crate::room::file_system::NodeFileSystem;
use async_trait::async_trait;
use meltos_tvn::file_system::FileSystem;

#[async_trait]
impl SessionConfigsIo for NodeFileSystem {
    async fn save(&self, session_configs: SessionConfigs) -> crate::error::Result {
        self.write(
            &self.path(".session"),
            &serde_json::to_vec(&session_configs).unwrap(),
        )?;
        Ok(())
    }

    async fn load(&self) -> crate::error::Result<SessionConfigs> {
        let buff = self.try_read(&self.path(".session"))?;
        Ok(serde_json::from_slice(&buff).unwrap())
    }
}
