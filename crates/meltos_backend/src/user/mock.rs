use std::collections::HashMap;

use async_trait::async_trait;

use crate::error;
use meltos::user::{SessionId, UserId};
use meltos_util::sync::arc_mutex::ArcMutex;

use crate::user::SessionIo;

#[derive(Debug, Default, Clone)]
pub struct MockUserSessionIo(ArcMutex<HashMap<SessionId, UserId>>);


impl MockUserSessionIo{
    pub async fn with_mock_users() -> Self{
        let me = Self::default();
        me.force_register(SessionId("owner".to_string()), UserId::from("owner")).await;
        me.force_register(SessionId("user".to_string()), UserId::from("owner")).await;
        me
    }
    
    pub async fn force_register(&self, session_id: SessionId, user_id: UserId){
        self.0.lock().await.insert(session_id, user_id);
    }
}

#[async_trait]
impl SessionIo for MockUserSessionIo {
    async fn fetch_user_id(&self, user_token: SessionId) -> crate::error::Result<UserId> {
        self.0
            .lock()
            .await
            .get(&user_token)
            .cloned()
            .ok_or(error::Error::UserIdNotExists)
    }

    async fn register(&self, user_token: SessionId, user_id: UserId) -> crate::error::Result {
        self.0.lock().await.insert(user_token, user_id);
        Ok(())
    }
}
