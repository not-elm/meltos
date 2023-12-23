use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageId, MessageText};
use meltos::discussion::{Discussion, DiscussionMeta};
use meltos::error;
use meltos::user::UserId;
use meltos_util::macros::Deref;


use crate::discussion::DiscussionIo;
use crate::sync::arc_mutex::ArcHashMap;

#[derive(Debug, Default)]
pub struct MockGlobalDiscussionIo {
    discussions: Discussions,
    messages: Messages,
    reply_discussions: ReplyDiscussions,
}

#[async_trait::async_trait]
impl DiscussionIo for MockGlobalDiscussionIo {
    async fn new_discussion(&self, creator: UserId) -> error::Result<DiscussionMeta> {
        let discussion = Discussion::new(creator);
        let mut discussions = self.discussions.lock().await;
        let meta = discussion.meta.clone();
        discussions.insert(meta.id.clone(), discussion);
        Ok(meta)
    }

    async fn speak(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        text: MessageText,
    ) -> error::Result<Message> {
        let message = Message::new(user_id, text);

        let mut discussions = self.discussions.lock().await;
        let discussion = discussions
            .get_mut(discussion_id)
            .ok_or(error::Error::DiscussionNotExists(discussion_id.clone()))?;
        discussion.messages.push(message.id.clone());

        let mut messages = self.messages.lock().await;
        messages.insert(message.id.clone(), message.clone());

        Ok(message)
    }

    async fn reply(
        &self,
        user_id: UserId,
        message_id: MessageId,
        text: MessageText,
    ) -> error::Result<Message> {
        let reply = Message::new(user_id, text);
        let mut discussion = self.reply_discussions.lock().await;
        if !discussion.contains_key(&message_id) {
            discussion.insert(message_id.clone(), vec![]);
        }

        discussion
            .get_mut(&message_id)
            .unwrap()
            .push(reply.id.clone());
        Ok(reply)
    }

    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<Discussion> {
        let mut discussions = self.discussions.lock().await;
        Ok(discussions.get_mut(discussion_id).unwrap().clone())
    }

    async fn all_discussions(&self) -> error::Result<Vec<Discussion>> {
        let discussions = self.discussions.lock().await;
        Ok(discussions.values().cloned().collect())
    }

    async fn close(&self, discussion_id: &DiscussionId) -> error::Result {
        let mut discussions = self.discussions.lock().await;
        let discussion = discussions
            .get_mut(discussion_id)
            .ok_or(error::Error::DiscussionNotExists(discussion_id.clone()))?;
        let message_ids = discussion.messages.clone();

        let mut reply_discussions = self.reply_discussions.lock().await;
        for id in message_ids {
            reply_discussions.remove(&id);
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Deref)]
struct Discussions(ArcHashMap<DiscussionId, Discussion>);

#[derive(Debug, Default, Clone, Deref)]
struct Messages(ArcHashMap<MessageId, Message>);

#[derive(Debug, Default, Clone, Deref)]
struct ReplyDiscussions(ArcHashMap<MessageId, Vec<MessageId>>);
