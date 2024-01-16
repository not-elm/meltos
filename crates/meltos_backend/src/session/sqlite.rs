use std::path::PathBuf;

use async_trait::async_trait;
use rusqlite::{Connection, params};
use tokio::sync::Mutex;

use meltos::room::RoomId;
use meltos::user::{SessionId, UserId};
use meltos_util::fs::delete_dir;

use crate::error;
use crate::error::Error::UserIdNotExists;
use crate::path::{create_resource_dir, room_resource_dir};
use crate::session::{NewSessionIo, SessionIo};

#[derive(Debug)]
pub struct SqliteSessionIo {
    db: Mutex<Connection>,
}


fn create_session_table(db: &Connection) -> rusqlite::Result<usize> {
    db.execute(
        "
            CREATE TABLE session(
            session_id TEXT NOT NULL PRIMARY KEY,
            user_id TEXT NOT NULL
            )
        ", ())
}


#[inline(always)]
fn delete_database(room_id: &RoomId) -> std::io::Result<()> {
    delete_dir(database_path(room_id))
}


#[inline(always)]
fn database_path(room_id: &RoomId) -> PathBuf {
    room_resource_dir(room_id).join("session.db")
}


impl NewSessionIo for SqliteSessionIo {
    fn new(room_id: RoomId) -> error::Result<Self> {
        delete_database(&room_id)?;
        create_resource_dir(&room_id)?;

        let db = Connection::open(database_path(&room_id))?;
        create_session_table(&db)?;

        Ok(Self {
            db: Mutex::new(db)
        })
    }
}

#[async_trait]
impl SessionIo for SqliteSessionIo {
    async fn register(&self, user_id: Option<UserId>) -> crate::error::Result<(UserId, SessionId)> {
        let session_id = SessionId::new();
        let user_id = user_id.unwrap_or_default();
        let db = self.db.lock().await;
        db.execute(
            "INSERT INTO session(session_id, user_id) VALUES($1, $2)",
            params![session_id.to_string(), user_id.to_string()],
        )?;

        Ok((user_id, session_id))
    }

    async fn unregister(&self, user_id: UserId) -> crate::error::Result {
        let db = self.db.lock().await;
        db.execute(
            "DELETE FROM session WHERE user_id=$1",
            params![user_id.0],
        )?;
        Ok(())
    }

    async fn fetch(&self, session_id: SessionId) -> crate::error::Result<UserId> {
        let db = self.db.lock().await;
        let result = db.query_row(
            "SELECT user_id FROM session WHERE session_id=$1",
            params![session_id.0],
            |row| Ok(UserId(row.get(0).unwrap())),
        );
        match result {
            Ok(user_id) => Ok(user_id),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(UserIdNotExists)
            }
            Err(e) => {
                Err(error::Error::Sqlite(e))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::future::Future;

    use futures::FutureExt;

    use meltos::room::RoomId;
    use meltos::user::{SessionId, UserId};

    use crate::error;
    use crate::session::{NewSessionIo, SessionIo};
    use crate::session::sqlite::{delete_database, SqliteSessionIo};

    #[tokio::test]
    async fn created_owner_id() {
        try_execute(|db| async move {
            let (user_id, session_id) = db.register(Some(UserId::from("user"))).await?;
            assert_eq!(user_id, UserId::from("user"));

            let fetched_user_id = db.fetch(session_id).await?;
            assert_eq!(fetched_user_id, UserId::from("user"));
            Ok(())
        })
            .await;
    }


    #[tokio::test]
    async fn deleted_user_id_after_unregister() {
        try_execute(|db| async move {
            let (user_id, session_id) = db.register(Some(UserId::from("user"))).await?;
            db.unregister(user_id).await?;
            assert!(db.fetch(session_id).await.is_err());
            Ok(())
        })
            .await;
    }


    #[tokio::test]
    async fn return_user_is_not_exists_error_when_unregistered() {
        try_execute(|db| async move {
            let result = db.fetch(SessionId::new()).await;
            assert!(matches!(result.unwrap_err(), error::Error::UserIdNotExists));
            Ok(())
        })
            .await;
    }

    async fn try_execute<F: Future<Output=crate::error::Result>>(f: impl FnOnce(SqliteSessionIo) -> F) {
        let room_id = RoomId::new();
        let db = SqliteSessionIo::new(room_id.clone()).unwrap();
        let result = std::panic::AssertUnwindSafe(f(db))
            .catch_unwind()
            .await;

        delete_database(&room_id).unwrap();
        result.unwrap().unwrap();
    }
}