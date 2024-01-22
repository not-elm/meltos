use async_trait::async_trait;
use auto_delegate::delegate;
use meltos::room::RoomId;

use meltos::user::{SessionId, UserId};

use crate::error;

pub mod mock;
pub mod sqlite;

pub trait NewSessionIo: Sized {
    fn new(room_id: RoomId) -> error::Result<Self>;
}

#[async_trait]
#[delegate]
pub trait SessionIo: Send + Sync {
    async fn register(&self, user_id: Option<UserId>) -> error::Result<(UserId, SessionId)>;

    async fn unregister(&self, user_id: UserId) -> error::Result;

    async fn fetch(&self, session_id: SessionId) -> error::Result<UserId>;
}
