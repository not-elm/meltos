use crate::discussion::structs::id::DiscussionId;
use crate::discussion::structs::message::{Message, MessageNo, MessageText};
use crate::discussion::structs::reply::Reply;
use crate::discussion::structs::{Discussion, DiscussionMeta};
use crate::error;
use crate::user::UserId;

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
    ) -> error::Result<Reply>;


    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<Discussion>;


    async fn all_discussions(&self) -> error::Result<Vec<Discussion>>;


    async fn close(&self, discussion_id: &DiscussionId) -> error::Result;
}
