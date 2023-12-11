use async_trait::async_trait;
use auto_delegate::delegate;
use meltos::user::{UserId, UserToken};

use crate::error;

pub mod mock;


#[async_trait]
#[delegate]
pub trait SessionIo: Send + Sync {
    async fn fetch_user_id(&self, session_token: UserToken) -> error::Result<UserId>;


    async fn register(&self, session_token: UserToken, user_id: UserId) -> error::Result;
}
