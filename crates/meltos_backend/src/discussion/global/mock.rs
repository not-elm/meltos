use meltos::discussion::{Discussion, DiscussionBundle, DiscussionMeta, MessageBundle};
use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageId, MessageText};
use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_util::macros::Deref;

use crate::discussion::{DiscussionIo, NewDiscussIo};
use crate::error;
use crate::sync::arc_mutex::ArcHashMap;

#[derive(Debug, Default)]
pub struct MockGlobalDiscussionIo {
    discussions: Discussions,
    messages: Messages,
    reply_discussions: ReplyDiscussions,
}


impl MockGlobalDiscussionIo {
    async fn message_bundles_in(&self, id: &DiscussionId) -> Option<Vec<MessageBundle>> {
        let discussions = self.discussions
            .lock()
            .await;

        let discussion_messages = &discussions.get(id)?.messages;
        let mut message_bundles = Vec::with_capacity(discussion_messages.len());
        for message_id in discussion_messages {
            if let Some(bundle) = self.message_bundle(message_id).await {
                message_bundles.push(bundle);
            }
        }
        Some(message_bundles)
    }

    async fn message_bundle(&self, id: &MessageId) -> Option<MessageBundle> {
        let reply = self.reply_discussions.lock().await;
        let messages = self.messages.lock().await;
        let message = messages.get(id)?.clone();

        if let Some(message_ids) = reply.get(id) {
            let mut replies = Vec::with_capacity(message_ids.len());

            for message_id in message_ids {
                replies.push(messages.get(message_id).unwrap().clone());
            }
            Some(MessageBundle {
                message,
                replies,
            })
        } else {
            Some(MessageBundle {
                message,
                replies: Vec::with_capacity(0),
            })
        }
    }
}


impl NewDiscussIo for MockGlobalDiscussionIo {
    fn new(_: RoomId) -> error::Result<Self> {
        Ok(Self::default())
    }
}

#[async_trait::async_trait]
impl DiscussionIo for MockGlobalDiscussionIo {
    async fn new_discussion(
        &self,
        title: String,
        creator: UserId,
    ) -> error::Result<DiscussionMeta> {
        let discussion = Discussion::new(title, creator);
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
        let message = Message::new(user_id.0, text.0);

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
        let reply = Message::new(user_id.0, text.0);
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

    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<DiscussionBundle> {
        let mut discussions = self.discussions.lock().await;
        let discussion = discussions.get_mut(discussion_id).unwrap();
        let meta = discussion.meta.clone();
        drop(discussions);

        Ok(DiscussionBundle {
            meta,
            messages: self.message_bundles_in(discussion_id).await.unwrap_or_default(),
        })
    }

    async fn all_discussions(&self) -> error::Result<Vec<DiscussionBundle>> {
        let discussions = self.discussions.lock().await;
        let mut bundles = Vec::new();
        let ids = discussions
            .iter()
            .map(|(id, _)|id)
            .cloned()
            .collect::<Vec<DiscussionId>>();
        drop(discussions);
        for id in ids {
            bundles.push(self.discussion_by(&id).await?);
        }
        Ok(bundles)
    }

    async fn close_discussion(&self, discussion_id: &DiscussionId) -> error::Result {
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
