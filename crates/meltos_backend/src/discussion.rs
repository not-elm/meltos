use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageNo, MessageText};
use meltos::discussion::reply::ReplyMessage;
use meltos::discussion::{Discussion, DiscussionMeta};
use meltos::error;
use meltos::user::UserId;

pub mod global;


#[async_trait::async_trait]
pub trait DiscussionIo: Send + Sync {
    async fn new_discussion(&self, creator: UserId) -> error::Result<DiscussionMeta>;


    async fn speak(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        message_text: MessageText,
    ) -> error::Result<Message>;


    async fn reply(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        message_no: MessageNo,
        message_text: MessageText,
    ) -> error::Result<ReplyMessage>;


    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<Discussion>;


    async fn all_discussions(&self) -> error::Result<Vec<Discussion>>;


    async fn close(&self, discussion_id: &DiscussionId) -> error::Result;
}
