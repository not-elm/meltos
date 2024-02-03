use meltos_core::discussion::id::DiscussionId;
use meltos_core::discussion::message::{Message, MessageId, MessageText};
use meltos_core::discussion::{DiscussionBundle, DiscussionMeta};
use meltos_core::room::RoomId;
use meltos_core::user::UserId;

use crate::error;

pub mod global;

pub trait NewDiscussIo: Sized {
    fn new(room_id: RoomId) -> error::Result<Self>;
}

#[async_trait::async_trait]
pub trait DiscussionIo: Send + Sync {
    async fn new_discussion(&self, title: String, creator: UserId)
        -> error::Result<DiscussionMeta>;

    async fn speak(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        text: MessageText,
    ) -> error::Result<Message>;

    async fn reply(
        &self,
        discussion_id: DiscussionId,
        user_id: UserId,
        message_id: MessageId,
        text: MessageText,
    ) -> error::Result<Message>;

    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<DiscussionBundle>;

    async fn all_discussions(&self) -> error::Result<Vec<DiscussionBundle>>;

    async fn close_discussion(&self, discussion_id: &DiscussionId) -> error::Result;
}
