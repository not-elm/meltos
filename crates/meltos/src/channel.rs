use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error;
use crate::schema::room::Joined;
use crate::user::UserId;

#[async_trait]
pub trait ChannelMessageSendable: Send + Sync {
    type Error: std::error::Error;

    fn user_id(&self) -> &UserId;

    async fn send(&mut self, message: ChannelMessage) -> std::result::Result<(), Self::Error>;
}

#[async_trait]
pub trait ChannelMessageReadable {
    type Error: std::error::Error;
    fn user_id(&self) -> &UserId;

    async fn read(&mut self) -> std::result::Result<ChannelMessage, Self::Error>;
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub from: UserId,
    pub message: Message,
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Message {
    Joined(Joined),
}