use crate::error;
use crate::thread::structs::id::ThreadId;
use crate::thread::structs::message::{MessageNo, MessageText};
use crate::thread::structs::MessageThread;
use crate::user::UserId;


pub mod global;

#[async_trait::async_trait]
pub trait ThreadIo: Send + Sync {
    async fn new_thread(&self) -> error::Result<ThreadId>;


    async fn speak(
        &self,
        thread_id: &ThreadId,
        user_id: UserId,
        message_text: MessageText,
    ) -> error::Result;


    async fn reply(
        &self,
        thread_id: &ThreadId,
        user_id: UserId,
        message_no: MessageNo,
        message_text: MessageText,
    ) -> error::Result;


    async fn thread_by(&self, thread_id: &ThreadId) -> error::Result<MessageThread>;


    async fn thread_all(&self) -> error::Result<Vec<MessageThread>>;


    async fn close(&self, thread_id: &ThreadId) -> error::Result;
}
