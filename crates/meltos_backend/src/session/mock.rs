use std::collections::HashMap;

use async_trait::async_trait;
use meltos::room::RoomId;

use meltos::user::{SessionId, UserId};

use crate::error;
use crate::sync::arc_mutex::ArcMutex;
use crate::session::{NewSessionIo, SessionIo};

#[derive(Debug, Default, Clone)]
pub struct MockSessionIo(ArcMutex<HashMap<SessionId, UserId>>);

impl MockSessionIo {
    pub async fn with_mock_users() -> Self {
        let me = Self::default();
        me.force_register(SessionId("owner".to_string()), UserId::from("owner"))
            .await;
        me.force_register(SessionId("tvc".to_string()), UserId::from("owner"))
            .await;
        me
    }

    pub async fn force_register(&self, session_id: SessionId, user_id: UserId) {
        self.0.lock().await.insert(session_id, user_id);
    }
}

impl MockSessionIo {
    async fn generate_session_id(&self) -> SessionId {
        let map = self.0.lock().await;
        loop {
            let session_id = SessionId::new();
            if !map.contains_key(&session_id) {
                return session_id;
            }
        }
    }

    async fn generate_user_id(&self) -> UserId {
        let map = self.0.lock().await;
        loop {
            let user_id = UserId::new();
            if !map.values().any(|id| id == &user_id) {
                return user_id;
            }
        }
    }
}



impl NewSessionIo for MockSessionIo {
    #[inline(always)]
    fn new(_room_id: RoomId) -> error::Result<Self> {
        Ok(Self::default())
    }
}


#[async_trait]
impl SessionIo for MockSessionIo {
    async fn register(&self, user_id: Option<UserId>) -> crate::error::Result<(UserId, SessionId)> {
        let session_id = self.generate_session_id().await;
        if let Some(user_id) = user_id {
            self.0
                .lock()
                .await
                .insert(session_id.clone(), user_id.clone());
            Ok((user_id, session_id))
        } else {
            let random_user = self.generate_user_id().await;
            self.0
                .lock()
                .await
                .insert(session_id.clone(), random_user.clone());
            Ok((random_user, session_id))
        }
    }

    async fn unregister(&self, user_id: UserId) -> error::Result {
        let mut map = self.0.lock().await;
        if let Some((session_id, _)) = map.clone().iter().find(|(_, v)| v == &&user_id) {
            map.remove(session_id);
        }

        Ok(())
    }

    async fn fetch(&self, user_token: SessionId) -> crate::error::Result<UserId> {
        self.0
            .lock()
            .await
            .get(&user_token)
            .cloned()
            .ok_or(error::Error::UserIdNotExists)
    }
}
