use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use meltos_core::room::RoomId;
use meltos_core::user::{SessionId, UserId};

use crate::error;
use crate::session::{NewSessionIo, SessionIo};
use crate::sync::arc_mutex::ArcMutex;

#[derive(Debug)]
pub struct MockSessionIo {
    map: ArcMutex<HashMap<SessionId, UserId>>,
    create_count: AtomicUsize,
}

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
        self.map.lock().await.insert(session_id, user_id);
    }
}

impl MockSessionIo {
    async fn generate_session_id(&self) -> SessionId {
        let map = self.map.lock().await;
        loop {
            let session_id = SessionId::new();
            if !map.contains_key(&session_id) {
                return session_id;
            }
        }
    }
}

impl Default for MockSessionIo {
    #[inline(always)]
    fn default() -> Self {
        Self {
            map: ArcMutex::default(),
            create_count: AtomicUsize::new(1),
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
        let create_count = self.create_count.fetch_add(1, Ordering::Relaxed);

        let session_id = self.generate_session_id().await;
        if let Some(user_id) = user_id {
            let mut map = self.map.lock().await;
            if map.values().any(|user| user == &user_id) {
                Err(error::Error::UserIdConflict(user_id))
            } else {
                map.insert(session_id.clone(), user_id.clone());
                Ok((user_id, session_id))
            }
        } else {
            let random_user = UserId(format!("guest{create_count}"));
            self.map
                .lock()
                .await
                .insert(session_id.clone(), random_user.clone());
            Ok((random_user, session_id))
        }
    }

    async fn unregister(&self, user_id: UserId) -> error::Result {
        let mut map = self.map.lock().await;
        if let Some((session_id, _)) = map.clone().iter().find(|(_, v)| v == &&user_id) {
            map.remove(session_id);
        }

        Ok(())
    }

    async fn fetch(&self, user_token: SessionId) -> crate::error::Result<UserId> {
        self.map
            .lock()
            .await
            .get(&user_token)
            .cloned()
            .ok_or(error::Error::SessionIdNotExists)
    }

    #[inline(always)]
    async fn user_count(&self) -> error::Result<u64> {
        Ok(self.map.lock().await.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use meltos_core::user::UserId;

    use crate::error::Error;
    use crate::session::mock::MockSessionIo;
    use crate::session::SessionIo;

    #[tokio::test]
    async fn failed_if_conflicts_user_ids() {
        let session = MockSessionIo::default();
        let user_id = UserId::from("user1");
        session.register(Some(user_id.clone())).await.unwrap();
        match session.register(Some(user_id.clone())).await.unwrap_err() {
            Error::UserIdConflict(id) => assert_eq!(id, user_id),
            _ => panic!("expected conflicts user_ids but did not"),
        }
    }

    #[tokio::test]
    async fn create_guest_ids() {
        let session = MockSessionIo::default();

        let (user_id, _) = session.register(None).await.unwrap();
        assert_eq!(user_id, UserId::from("guest1"));

        let (user_id, _) = session.register(None).await.unwrap();
        assert_eq!(user_id, UserId::from("guest2"));

        let (user_id, _) = session.register(None).await.unwrap();
        assert_eq!(user_id, UserId::from("guest3"));
    }
}
