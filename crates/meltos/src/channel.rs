use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::schema::discussion::global::{Closed, Created, Replied, Spoke};
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
    pub message: MessageData,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageData {
    Joined { user_id: String },
    DiscussionCreated(Created),
    DiscussionSpoke(Spoke),
    DiscussionReplied(Replied),
    DiscussionClosed(Closed),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::channel::MessageData;

    #[test]
    fn json_type() {
        let message = MessageData::Joined {
            user_id: "user".to_string(),
        };
        let json = json!(message);
        let ty = json.get("type").unwrap();
        assert_eq!(ty.as_str(), Some("joined"))
    }
}
