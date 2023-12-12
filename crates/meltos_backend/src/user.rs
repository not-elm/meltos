use async_trait::async_trait;
use auto_delegate::delegate;
use meltos::user::{UserId, SessionId};

use crate::error;

pub mod mock;


#[async_trait]
#[delegate]
pub trait SessionIo: Send + Sync {
    async fn fetch_user_id(&self, session_token: SessionId) -> error::Result<UserId>;


    async fn register(&self, session_token: SessionId, user_id: UserId) -> error::Result;
}
