use std::collections::HashMap;
use std::sync::Arc;

use meltos_core::user::UserId;
use tokio::sync::{Mutex, MutexGuard};

use crate::error;
use crate::io::ThreadIo;
use crate::structs::id::ThreadId;
use crate::structs::message::{MessageNo, MessageText};
use crate::structs::Thread;

#[derive(Debug, Default)]
pub struct MockGlobalThreadIo(Arc<Mutex<HashMap<ThreadId, Thread>>>);


impl MockGlobalThreadIo {
    pub async fn lock_thread(
        &self,
        thread_id: &ThreadId,
    ) -> error::Result<MutexGuard<HashMap<ThreadId, Thread>>> {
        let map = self.0.lock().await;
        if !map.contains_key(thread_id) {
            Err(error::Error::ThreadNotExists(thread_id.clone()))
        } else {
            Ok(map)
        }
    }
}


#[async_trait::async_trait]
impl ThreadIo for MockGlobalThreadIo {
    async fn new_thread(&self) -> error::Result<ThreadId> {
        let thread = Thread::default();
        let mut thread_map = self.0.lock().await;
        let id = thread.id.clone();
        thread_map.insert(thread.id.clone(), thread);
        Ok(id)
    }


    async fn speak(
        &self,
        thread_id: &ThreadId,
        user_id: UserId,
        message_text: MessageText,
    ) -> error::Result {
        let mut map = self.lock_thread(thread_id).await?;
        map.get_mut(thread_id)
            .unwrap()
            .add_message(user_id, message_text);
        Ok(())
    }


    async fn reply(
        &self,
        thread_id: &ThreadId,
        user_id: UserId,
        message_no: MessageNo,
        message_text: MessageText,
    ) -> error::Result {
        let mut map = self.lock_thread(thread_id).await?;
        map.get_mut(thread_id)
            .unwrap()
            .add_reply(user_id, message_no, message_text)
    }


    async fn thread_by(&self, thread_id: &ThreadId) -> error::Result<Thread> {
        let mut map = self.lock_thread(thread_id).await?;
        Ok(map.get_mut(thread_id).unwrap().clone())
    }


    async fn thread_all(&self) -> error::Result<Vec<Thread>> {
        let map = self.0.lock().await;
        Ok(map.values().cloned().collect())
    }


    async fn close(&self, thread_id: &ThreadId) -> error::Result {
        let mut map = self.0.lock().await;
        let thread = map
            .get_mut(thread_id)
            .ok_or(crate::error::Error::ThreadNotExists(thread_id.clone()))?;
        thread.messages.clear();
        Ok(())
    }
}


impl Clone for MockGlobalThreadIo {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
