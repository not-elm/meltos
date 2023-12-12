use crate::error;
use async_trait::async_trait;
use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::Message;
use meltos::discussion::reply::ReplyMessage;
use meltos::discussion::DiscussionMeta;

#[async_trait]
pub trait ClientDiscussionIo: Send + Sync {
    async fn created(&self, discussion_meta: DiscussionMeta) -> error::Result;


    async fn spoke(&self, discussion_id: DiscussionId, messages: Message) -> error::Result;


    async fn replied(&self, discussion_id: DiscussionId, reply: ReplyMessage) -> error::Result;


    async fn closed(&self, discussion_id: DiscussionId) -> error::Result;
}
