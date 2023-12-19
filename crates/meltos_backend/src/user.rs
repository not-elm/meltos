use async_trait::async_trait;
use auto_delegate::delegate;

use meltos::user::{SessionId, UserId};

use crate::error;

pub mod mock;

#[async_trait]
#[delegate]
pub trait SessionIo: Send + Sync {
    async fn register(&self, user_id: Option<UserId>) -> error::Result<(UserId, SessionId)>;


    async fn fetch(&self, session_id: SessionId) -> error::Result<UserId>;
}
