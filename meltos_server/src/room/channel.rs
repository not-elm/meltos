use axum::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;

use meltos_core::channel::{ChannelMessage, ChannelMessageSendable};
use meltos_core::user::UserId;

pub struct WebsocketSender {
    pub user_id: UserId,
    pub tx: SplitSink<WebSocket, Message>,
}

impl WebsocketSender {
    #[inline]
    pub const fn new(user_id: UserId, tx: SplitSink<WebSocket, Message>) -> WebsocketSender {
        Self {
            user_id,
            tx,
        }
    }
}

#[async_trait]
impl ChannelMessageSendable for WebsocketSender {
    type Error = crate::error::Error;

    #[inline(always)]
    fn user_id(&self) -> &UserId {
        &self.user_id
    }

    async fn send(&mut self, message: ChannelMessage) -> crate::error::Result {
        self.tx
            .send(Message::Text(serde_json::to_string(&message)?))
            .await
            .map_err(|e| crate::error::Error::FailedSendChannelMessage(e.to_string()))?;
        Ok(())
    }
}
