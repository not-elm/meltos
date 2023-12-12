use meltos::discussion::{Discussion, DiscussionMeta};
use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageNo, MessageText};
use meltos::discussion::reply::Reply;
use meltos::error;
use meltos::user::UserId;

use crate::discussion::global::mock::MockGlobalDiscussionIo;

pub mod global;


#[derive(Default, Debug, Clone, Copy, Hash)]
pub struct MockDiscussionIoTypes;

impl DiscussionIoTypes for MockDiscussionIoTypes {
    type Global = MockGlobalDiscussionIo;
}


pub trait DiscussionIoTypes: Default + Send + Sync + 'static {
    type Global: DiscussionIo;
}


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
