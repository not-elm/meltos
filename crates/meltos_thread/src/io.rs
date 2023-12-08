use crate::error;
use crate::structs::id::ThreadId;
use crate::structs::message::{MessageNo, MessageText};
use crate::structs::Thread;
use meltos_core::user::UserId;

pub mod global;

#[async_trait::async_trait]
#[auto_delegate::delegate]
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


    async fn thread_by(&self, thread_id: &ThreadId) -> error::Result<Thread>;


    async fn thread_all(&self) -> error::Result<Vec<Thread>>;


    async fn close(&self, thread_id: &ThreadId) -> error::Result;
}
