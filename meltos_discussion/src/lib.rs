use crate::thread::message::{MessageNo, MessageText};
use crate::thread::{Thread, ThreadId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub mod error;
pub mod thread;

#[async_trait::async_trait]
#[auto_delegate::delegate]
pub trait ThreadIo: Send + Sync {
    async fn new_thread(&self, file_path: impl Into<String> + Send, file_line_no: usize);

    async fn speak(&self, thread_id: &ThreadId, message_text: MessageText) -> error::Result;

    async fn reply(
        &self,
        thread_id: &ThreadId,
        message_no: &MessageNo,
        message_text: MessageText,
    ) -> error::Result;

    async fn close(&self, thread_id: &ThreadId) -> error::Result;
}


#[derive(Debug, Default)]
pub struct Discussions(Arc<Mutex<HashMap<ThreadId, ThreadData>>>);

impl Discussions {
    pub async fn lock_thread(
        &self,
        thread_id: &ThreadId,
    ) -> error::Result<MutexGuard<HashMap<ThreadId, ThreadData>>> {
        let map = self.0.lock().await;
        if !map.contains_key(thread_id) {
            Err(error::Error::ThreadNotExists(thread_id.clone()))
        } else {
            Ok(map)
        }
    }
}


#[async_trait::async_trait]
impl ThreadIo for Discussions {
    async fn new_thread(&self, file_path: impl Into<String> + Send, file_line_no: usize) {
        let thread_data = ThreadData::new(file_path.into(), file_line_no);
        let mut thread_map = self.0.lock().await;
        thread_map.insert(thread_data.thread_id(), thread_data);
    }

    async fn speak(&self, thread_id: &ThreadId, message_text: MessageText) -> error::Result {
        let mut map = self.lock_thread(thread_id).await?;
        map.get_mut(thread_id).unwrap().thread.speak(message_text);
        Ok(())
    }

    async fn reply(
        &self,
        thread_id: &ThreadId,
        message_no: &MessageNo,
        message_text: MessageText,
    ) -> error::Result {
        todo!()
    }

    async fn close(&self, thread_id: &ThreadId) -> error::Result {
        todo!()
    }
}


impl Clone for Discussions {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


#[derive(Debug)]
pub struct ThreadData {
    pub file_path: String,
    pub thread: Thread,
}


impl ThreadData {
    #[inline]
    pub fn new(file_path: String, file_line_no: usize) -> Self {
        Self {
            file_path,
            thread: Thread::new(file_line_no),
        }
    }

    #[inline]
    pub fn thread_id(&self) -> ThreadId {
        self.thread.id.clone()
    }
}
