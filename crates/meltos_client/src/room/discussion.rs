use async_trait::async_trait;

use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::Message;
use meltos::discussion::DiscussionMeta;

#[async_trait]
pub trait ClientDiscussionIo: Send + Sync {
    type Error: std::error::Error;


    async fn created(&self, discussion_meta: DiscussionMeta) -> Result<(), Self::Error>;

    async fn spoke(
        &self,
        discussion_id: DiscussionId,
        messages: Message,
    ) -> Result<(), Self::Error>;

    async fn replied(&self, discussion_id: DiscussionId, reply: Message)
        -> Result<(), Self::Error>;

    async fn closed(&self, discussion_id: DiscussionId) -> Result<(), Self::Error>;
}
