use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use meltos::discussion::{Discussion, DiscussionMeta};
use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageNo, MessageText};
use meltos::discussion::reply::Reply;
use meltos::error;
use meltos::user::UserId;
use crate::discussion::DiscussionIo;

#[derive(Debug, Default)]
pub struct MockGlobalDiscussionIo(Arc<Mutex<HashMap<DiscussionId, Discussion>>>);


impl MockGlobalDiscussionIo {
    pub async fn lock_thread(
        &self,
        discussion_id: &DiscussionId,
    ) -> error::Result<MutexGuard<HashMap<DiscussionId, Discussion>>> {
        let map = self.0.lock().await;
        if !map.contains_key(discussion_id) {
            Err(error::Error::ThreadNotExists(discussion_id.clone()))
        } else {
            Ok(map)
        }
    }
}


#[async_trait::async_trait]
impl DiscussionIo for MockGlobalDiscussionIo {
    async fn new_discussion(&self, creator: UserId) -> error::Result<DiscussionMeta> {
        let thread = Discussion::new(creator);
        let mut thread_map = self.0.lock().await;
        let meta = thread.meta.clone();
        thread_map.insert(meta.id.clone(), thread);
        Ok(meta)
    }


    async fn speak(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        message_text: MessageText,
    ) -> error::Result<Message> {
        let mut map = self.lock_thread(discussion_id).await?;
        let message = map
            .get_mut(discussion_id)
            .unwrap()
            .add_message(user_id, message_text);
        Ok(message)
    }


    async fn reply(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        message_no: MessageNo,
        message_text: MessageText,
    ) -> error::Result<Reply> {
        let mut map = self.lock_thread(discussion_id).await?;
        map.get_mut(discussion_id)
            .unwrap()
            .add_reply(user_id, message_no, message_text)
    }


    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<Discussion> {
        let mut map = self.lock_thread(discussion_id).await?;
        Ok(map.get_mut(discussion_id).unwrap().clone())
    }


    async fn all_discussions(&self) -> error::Result<Vec<Discussion>> {
        let map = self.0.lock().await;
        Ok(map.values().cloned().collect())
    }


    async fn close(&self, discussion_id: &DiscussionId) -> error::Result {
        let mut map = self.0.lock().await;
        let thread = map
            .get_mut(discussion_id)
            .ok_or(meltos::error::Error::ThreadNotExists(discussion_id.clone()))?;
        thread.messages.clear();
        Ok(())
    }
}


impl Clone for MockGlobalDiscussionIo {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
