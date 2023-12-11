use std::collections::HashMap;

use async_trait::async_trait;

use meltos::user::{UserToken, UserId};
use meltos_util::sync::arc_mutex::ArcMutex;
use crate::error;

use crate::user::UserSessionIo;


#[derive(Debug, Default, Clone)]
pub struct MockUserSessionIo(ArcMutex<HashMap<UserToken, UserId>>);


#[async_trait]
impl UserSessionIo for MockUserSessionIo {
    async fn fetch_user_id(&self, user_token: UserToken) -> crate::error::Result<UserId> {
        self.0.lock().await.get(&user_token).cloned().ok_or(error::Error::UserIdNotExists)
    }

    async fn register(&self, user_token: UserToken, user_id: UserId) -> crate::error::Result {
        self.0.lock().await.insert(user_token, user_id);
        Ok(())
    }
}