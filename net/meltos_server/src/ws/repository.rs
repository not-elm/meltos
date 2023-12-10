pub mod hash_map;

use crate::error;
use crate::ws::sender::CommandSender;
use async_trait::async_trait;
use meltos::session::SessionId;
use meltos::user::UserId;

#[async_trait]
pub trait CommandSenderRepository: Send + Sync + Clone {
    async fn fetch_all_senders(
        &self,
        session_id: &SessionId,
    ) -> error::Result<&mut [CommandSender]>;


    async fn fetch_sender(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
    ) -> error::Result<&mut CommandSender>;


    async fn insert_sender(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
        websocket_sender: CommandSender,
    ) -> error::Result;


    async fn remove_sender(&self, session_id: &SessionId, user_id: &UserId) -> error::Result;
}
