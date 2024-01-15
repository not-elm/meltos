use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use rusqlite::{Connection, params};

use meltos::discussion::{Discussion, DiscussionMeta};
use meltos::discussion::id::DiscussionId;
use meltos::discussion::message::{Message, MessageId, MessageText};
use meltos::room::RoomId;
use meltos::user::UserId;

use crate::discussion::DiscussionIo;
use crate::error;

pub struct SqliteDiscussionIo {
    db: Mutex<rusqlite::Connection>,
}


impl SqliteDiscussionIo {
    pub fn new(room_id: &RoomId) -> error::Result<Self> {
        let path = format!("./{room_id}.db");
        if std::fs::metadata(&path).is_ok() {
            std::fs::remove_file(&path)?;
        }
        let db = rusqlite::Connection::open(&path)?;

        create_discussion_meta_table(&db)?;
        create_message_table(&db)?;
        create_discussion_message_table(&db)?;
        create_reply_message_table(&db)?;

        Ok(Self {
            db: Mutex::new(db)
        })
    }


    fn lock(&self) -> MutexGuard<Connection> {
        self.db.lock().unwrap()
    }
}

fn create_discussion_meta_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute("CREATE TABLE discussion_meta (
            discussion_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            creator TEXT NOT NULL
        )", ())
}

fn create_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute("CREATE TABLE message (
            message_id TEXT PRIMARY KEY,
            user_id TEXT,
            text TEXT
            )", ())
}

fn create_discussion_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute("CREATE TABLE discussion_message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            discussion_id TEXT,
            message_id TEXT
            )", ())
}

fn create_reply_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute("CREATE TABLE reply_message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT,
            reply_message_id TEXT
            )", ())
}


#[async_trait]
impl DiscussionIo for SqliteDiscussionIo {
    async fn new_discussion(&self, title: String, creator: UserId) -> error::Result<DiscussionMeta> {
        let db = self.lock();
        let meta = DiscussionMeta::new(DiscussionId::new(), title, creator);
        db.execute(
            "INSERT INTO discussion_meta(discussion_id, title, creator) VALUES(?1, ?2, ?3)",
            params![meta.id.to_string(), meta.title, meta.creator.to_string()],
        )?;
        Ok(meta)
    }

    async fn speak(&self, discussion_id: &DiscussionId, user_id: UserId, text: MessageText) -> error::Result<Message> {
        let db = self.lock();
        let message = Message::new(user_id.0, text.0);
        db.execute(
            "INSERT INTO message(message_id, user_id, text) VALUES(?1, ?2, ?3)",
            params![message.id.to_string(), message.user_id.to_string(), message.text.to_string()],
        )?;
        db.execute(
            "INSERT INTO discussion_message(discussion_id, message_id) VALUES(?1, ?2)",
            params![discussion_id.to_string(), message.id.to_string()],
        )?;

        Ok(message)
    }

    async fn reply(&self, user_id: UserId, to: MessageId, text: MessageText) -> error::Result<Message> {
        let db = self.lock();
        let message = Message::new(user_id.to_string(), text.to_string());
        db.execute(
            "INSERT INTO message(message_id, user_id, text) VALUES(?1, ?2, ?3)",
            params![message.id.to_string(), message.user_id.to_string(), message.text.to_string()],
        )?;
        db.execute(
            "INSERT INTO reply_message(message_id, reply_message_id) VALUES(?1, ?2)",
            params![to.to_string(), message.id.to_string()],
        )?;

        Ok(message)
    }

    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<Discussion> {
        todo!()
    }

    async fn all_discussions(&self) -> error::Result<Vec<Discussion>> {
        todo!()
    }

    async fn close_discussion(&self, discussion_id: &DiscussionId) -> error::Result {
        todo!()
    }

    async fn dispose(self) -> error::Result {
        let path = {
            let db = self.db.lock().unwrap();
            db.path().unwrap().to_string()
        };
        let db = self.db.into_inner().unwrap();
        db.close().map_err(|(_, error)| error)?;
        if std::fs::metadata(&path).is_ok() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::future::Future;

    use futures::FutureExt;

    use meltos::discussion::message::MessageText;
    use meltos::room::RoomId;
    use meltos::user::UserId;

    use crate::discussion::DiscussionIo;
    use crate::discussion::global::sqlite::SqliteDiscussionIo;
    use crate::error;

    #[tokio::test]
    async fn success_create_tables() {
        let db = create_db();
        db.dispose().await.unwrap();
    }

    #[tokio::test]
    async fn create_discussion_meta() {
        try_execute(|db| async move {
            let meta = db.new_discussion("title".to_string(), UserId::from("user")).await?;
            assert_eq!(meta.id.0.len(), 40);
            assert_eq!(meta.creator, UserId::from("user"));
            assert_eq!(meta.title, "title".to_string());
            Ok(())
        })
            .await;
    }


    #[tokio::test]
    async fn speak_message() {
        try_execute(|db| async move {
            let meta = db.new_discussion("title".to_string(), UserId::from("user")).await?;
            let message = db.speak(&meta.id, UserId::from("user2"), MessageText::from("hello world!")).await?;
            assert_eq!(message.id.0.len(), 40);
            assert_eq!(message.user_id, UserId::from("user2"));
            assert_eq!(&message.text.0, "hello world!");
            Ok(())
        })
            .await;
    }

    #[tokio::test]
    async fn reply_message() {
        try_execute(|db| async move {
            let meta = db.new_discussion("title".to_string(), UserId::from("user")).await?;
            let message = db.speak(&meta.id, UserId::from("user2"), MessageText::from("hello world!")).await?;
            let reply = db.reply(UserId::from("user"), message.id, MessageText::from("reply")).await?;
            assert_eq!(reply.id.0.len(), 40);
            assert_eq!(reply.user_id, UserId::from("user"));
            assert_eq!(&reply.text.0, "reply");
            Ok(())
        })
            .await;
    }

    async fn try_execute<F: Future<Output=error::Result>>(f: impl FnOnce(SqliteDiscussionIo) -> F + 'static + Send) {
        let db = create_db();
        let path = db.db.lock().unwrap().path().unwrap().to_string();
        let result = std::panic::AssertUnwindSafe(f(db)).catch_unwind().await;
        if std::fs::metadata(&path).is_ok() {
            std::fs::remove_file(&path).unwrap();
        }
        let _ = result.expect("execute error");
    }


    fn create_db() -> SqliteDiscussionIo {
        SqliteDiscussionIo::new(&RoomId::new()).unwrap()
    }
}