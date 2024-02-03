use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use async_trait::async_trait;
use rusqlite::{params, Connection, Transaction};

use meltos_core::discussion::id::DiscussionId;
use meltos_core::discussion::message::{Message, MessageId, MessageText};
use meltos_core::discussion::{DiscussionBundle, DiscussionMeta, MessageBundle};
use meltos_core::room::RoomId;
use meltos_core::user::UserId;

use crate::discussion::{DiscussionIo, NewDiscussIo};
use crate::error;
use crate::path::{create_resource_dir, room_resource_dir};

#[derive(Debug)]
pub struct SqliteDiscussionIo {
    db: Mutex<Connection>,
}

impl SqliteDiscussionIo {
    fn lock(&self) -> MutexGuard<Connection> {
        self.db.lock().unwrap()
    }

    fn read_discussion_ids(&self) -> error::Result<Vec<DiscussionId>> {
        let db = self.lock();
        let mut stmt = db.prepare("SELECT discussion_id FROM discussion_meta")?;
        let rows = stmt.query_map((), |row| Ok(DiscussionId(row.get(0).unwrap())))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }
        Ok(ids)
    }

    fn exists_discussion(id: DiscussionId, tx: &Transaction) -> error::Result<bool> {
        let exists = tx.query_row(
            "SELECT EXISTS(SELECT discussion_id FROM discussion_meta WHERE discussion_id = ?1)",
            params![id.0],
            |row| {
                let n: isize = row.get(0).unwrap();
                Ok(n == 1)
            },
        )?;
        Ok(exists)
    }

    fn exists_message(id: MessageId, tx: &Transaction) -> error::Result<bool> {
        let exists = tx.query_row(
            "SELECT EXISTS(SELECT message_id FROM message WHERE message_id = ?1)",
            params![id.0],
            |row| {
                let n: isize = row.get(0).unwrap();
                Ok(n == 1)
            },
        )?;
        Ok(exists)
    }
}

impl NewDiscussIo for SqliteDiscussionIo {
    fn new(room_id: RoomId) -> error::Result<Self> {
        delete_database_if_exists(&room_id)?;
        create_resource_dir(&room_id)?;

        let db = rusqlite::Connection::open(database_path(&room_id))?;

        create_discussion_meta_table(&db)?;
        create_message_table(&db)?;
        create_message_table_trigger(&db)?;
        create_discussion_message_table(&db)?;
        create_reply_message_table(&db)?;
        create_discussion_message_table_trigger(&db)?;

        Ok(Self {
            db: Mutex::new(db),
        })
    }
}

#[async_trait]
impl DiscussionIo for SqliteDiscussionIo {
    async fn new_discussion(
        &self,
        title: String,
        creator: UserId,
    ) -> error::Result<DiscussionMeta> {
        let db = self.lock();
        let meta = DiscussionMeta::new(DiscussionId::new(), title, creator);
        db.execute(
            "INSERT INTO discussion_meta(discussion_id, title, creator) VALUES(?1, ?2, ?3)",
            params![meta.id.to_string(), meta.title, meta.creator.to_string()],
        )?;
        Ok(meta)
    }

    async fn speak(
        &self,
        discussion_id: &DiscussionId,
        user_id: UserId,
        text: MessageText,
    ) -> error::Result<Message> {
        let mut db = self.lock();
        let message = Message::new(user_id.0, text.0);
        let tx = db.transaction()?;
        if !Self::exists_discussion(discussion_id.clone(), &tx)? {
            return Err(error::Error::DiscussionNotExists(discussion_id.clone()));
        }

        tx.execute(
            "INSERT INTO message(message_id, user_id, text) VALUES(?1, ?2, ?3)",
            params![
                message.id.to_string(),
                message.user_id.to_string(),
                message.text.to_string()
            ],
        )?;
        tx.execute(
            "INSERT INTO discussion_message(discussion_id, message_id) VALUES(?1, ?2)",
            params![discussion_id.to_string(), message.id.to_string()],
        )?;
        tx.commit()?;
        Ok(message)
    }

    async fn reply(
        &self,
        discussion_id: DiscussionId,
        user_id: UserId,
        to: MessageId,
        text: MessageText,
    ) -> error::Result<Message> {
        let mut db = self.lock();
        let tx = db.transaction()?;
        if !Self::exists_discussion(discussion_id.clone(), &tx)? {
            return Err(error::Error::DiscussionNotExists(discussion_id));
        }
        if !Self::exists_message(to.clone(), &tx)? {
            return Err(error::Error::MessageNotExists(to));
        }

        let message = Message::new(user_id.to_string(), text.to_string());
        tx.execute(
            "INSERT INTO message(message_id, user_id, text) VALUES(?1, ?2, ?3)",
            params![
                message.id.to_string(),
                message.user_id.to_string(),
                message.text.to_string()
            ],
        )?;
        tx.execute(
            "INSERT INTO reply_message(message_id, reply_message_id) VALUES(?1, ?2)",
            params![to.to_string(), message.id.to_string()],
        )?;
        tx.commit()?;

        Ok(message)
    }

    async fn discussion_by(&self, discussion_id: &DiscussionId) -> error::Result<DiscussionBundle> {
        let mut db = self.lock();
        let tx = db.transaction()?;
        let meta = read_discussion_meta(&tx, discussion_id.clone())?;
        let messages = read_messages_in(&tx, discussion_id.clone())?;
        tx.commit()?;

        Ok(DiscussionBundle {
            meta,
            messages,
        })
    }

    async fn all_discussions(&self) -> error::Result<Vec<DiscussionBundle>> {
        let ids = self.read_discussion_ids()?;
        let mut discussions = Vec::with_capacity(ids.len());
        for id in ids {
            discussions.push(self.discussion_by(&id).await?);
        }
        Ok(discussions)
    }

    async fn close_discussion(&self, discussion_id: &DiscussionId) -> error::Result {
        let db = self.lock();
        db.execute(
            "DELETE FROM discussion_meta WHERE discussion_id = $1",
            params![discussion_id.to_string()],
        )?;
        Ok(())
    }
}

#[inline(always)]
fn database_path(room_id: &RoomId) -> PathBuf {
    room_resource_dir(room_id).join("discussion.db")
}

fn delete_database_if_exists(room_id: &RoomId) -> std::io::Result<()> {
    let path = database_path(room_id);
    if std::fs::metadata(&path).is_ok() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

fn create_discussion_meta_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE discussion_meta (
            discussion_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            creator TEXT NOT NULL
        )",
        (),
    )
}

fn create_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE message (
            message_id TEXT PRIMARY KEY,
            user_id TEXT,
            text TEXT
            )",
        (),
    )
}

fn create_discussion_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE discussion_message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            discussion_id TEXT,
            message_id TEXT,
            FOREIGN KEY(message_id) REFERENCES message(message_id)
            )",
        (),
    )
}

fn create_reply_message_table(db: &rusqlite::Connection) -> rusqlite::Result<usize> {
    db.execute(
        "CREATE TABLE reply_message (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT,
            reply_message_id TEXT,
            FOREIGN KEY(message_id) REFERENCES message(message_id),
            FOREIGN KEY(reply_message_id) REFERENCES message(message_id)
    )",
        (),
    )
}

fn create_message_table_trigger(db: &Connection) -> rusqlite::Result<usize> {
    db.execute(
        "
            CREATE TRIGGER delete_message_trigger DELETE ON discussion_meta
            BEGIN
            DELETE FROM message
            WHERE message_id IN(
                SELECT message.message_id FROM message
                LEFT JOIN discussion_message
                    ON (message.message_id=discussion_message.message_id)
                WHERE discussion_message.discussion_id=old.discussion_id
            );

            DELETE FROM discussion_message WHERE discussion_id=old.discussion_id;

            END;
            ",
        (),
    )
}

fn create_discussion_message_table_trigger(db: &Connection) -> rusqlite::Result<usize> {
    db.execute(
        "
            CREATE TRIGGER delete_discussion_message_trigger DELETE ON message
            BEGIN
            DELETE FROM message
            WHERE message_id IN(
                SELECT reply_message.reply_message_id FROM reply_message
                LEFT JOIN message
                    ON (message.message_id=reply_message.message_id)
                WHERE reply_message.message_id=old.message_id
            );
            DELETE FROM reply_message WHERE message_id=old.message_id;

            END;
            ",
        (),
    )
}

fn read_discussion_meta(tx: &Transaction, id: DiscussionId) -> error::Result<DiscussionMeta> {
    let meta: DiscussionMeta = tx.query_row(
        "SELECT title, creator FROM discussion_meta WHERE discussion_id LIKE $1",
        params![id.to_string()],
        |row| {
            Ok(DiscussionMeta {
                id,
                title: row.get(0).unwrap(),
                creator: UserId(row.get(1).unwrap()),
            })
        },
    )?;
    Ok(meta)
}

fn read_messages_in(tx: &Transaction, id: DiscussionId) -> error::Result<Vec<MessageBundle>> {
    let message_ids = read_message_ids_in(tx, id)?;
    let mut bundles = Vec::with_capacity(message_ids.len());
    let messages = read_all_messages(tx)?;
    for message in messages.iter().filter(|m| message_ids.contains(&m.id)) {
        let reply_message_ids = read_reply_message_ids_in(tx, message.id.clone())?;
        bundles.push(MessageBundle {
            message: message.clone(),
            replies: messages
                .iter()
                .filter(|m| reply_message_ids.contains(&m.id))
                .cloned()
                .collect(),
        })
    }
    Ok(bundles)
}

fn read_all_messages(tx: &Transaction) -> error::Result<Vec<Message>> {
    let mut stmt = tx.prepare("SELECT message_id, user_id, text FROM message")?;
    let rows = stmt.query_map((), |row| {
        Ok(Message {
            id: MessageId(row.get(0).unwrap()),
            user_id: UserId(row.get(1).unwrap()),
            text: MessageText(row.get(2).unwrap()),
        })
    })?;
    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    Ok(messages)
}

fn read_reply_message_ids_in(tx: &Transaction, id: MessageId) -> error::Result<Vec<MessageId>> {
    let mut stmt =
        tx.prepare("SELECT reply_message_id FROM reply_message WHERE message_id = $1")?;
    let message_id_rows = stmt.query_map(params![id.to_string()], |row| {
        Ok(MessageId(row.get(0).unwrap()))
    })?;

    let mut message_ids = Vec::new();
    for message_id in message_id_rows {
        message_ids.push(message_id?);
    }
    Ok(message_ids)
}

fn read_message_ids_in(tx: &Transaction, id: DiscussionId) -> error::Result<Vec<MessageId>> {
    let mut stmt =
        tx.prepare("SELECT message_id FROM discussion_message WHERE discussion_id = $1")?;
    let message_id_rows = stmt.query_map(params![id.to_string()], |row| {
        Ok(MessageId(row.get(0).unwrap()))
    })?;

    let mut message_ids = Vec::new();
    for message_id in message_id_rows {
        message_ids.push(message_id?);
    }
    Ok(message_ids)
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use futures::FutureExt;

    use meltos_core::discussion::id::DiscussionId;
    use meltos_core::discussion::message::{MessageId, MessageText};
    use meltos_core::discussion::MessageBundle;
    use meltos_core::room::RoomId;
    use meltos_core::user::UserId;

    use crate::discussion::global::sqlite::{
        read_all_messages, read_message_ids_in, read_reply_message_ids_in, SqliteDiscussionIo,
    };
    use crate::discussion::{DiscussionIo, NewDiscussIo};
    use crate::error;
    use crate::path::delete_resource_dir;

    #[tokio::test]
    async fn success_create_tables() {
        let room_id = RoomId::new();
        match SqliteDiscussionIo::new(room_id.clone()) {
            Ok(db) => {
                drop(db);
                delete_resource_dir(&room_id).unwrap();
            }
            Err(e) => {
                delete_resource_dir(&room_id).unwrap();
                panic!("{e}");
            }
        }
    }

    #[tokio::test]
    async fn create_discussion_meta() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;
                assert_eq!(meta.id.0.len(), 40);
                assert_eq!(meta.creator, UserId::from("session"));
                assert_eq!(meta.title, "title".to_string());
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn speak_message() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;
                let message = db
                    .speak(
                        &meta.id,
                        UserId::from("user2"),
                        MessageText::from("hello world!"),
                    )
                    .await?;
                assert_eq!(message.id.0.len(), 40);
                assert_eq!(message.user_id, UserId::from("user2"));
                assert_eq!(&message.text.0, "hello world!");
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn failed_speak_if_not_exists_discussion() {
        try_execute(|db| {
            async move {
                db.new_discussion("title".to_string(), UserId::from("session"))
                    .await?;

                match db
                    .speak(
                        &DiscussionId("ID".to_string()),
                        UserId::from("user2"),
                        MessageText::from("hello world!"),
                    )
                    .await
                {
                    Err(error::Error::DiscussionNotExists(id)) => {
                        assert_eq!(DiscussionId("ID".to_string()), id)
                    }
                    _ => panic!("expected DiscussionNotExists but was."),
                }
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn reply_message() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;
                let message = db
                    .speak(
                        &meta.id,
                        UserId::from("user2"),
                        MessageText::from("hello world!"),
                    )
                    .await?;
                let reply = db
                    .reply(
                        meta.id,
                        UserId::from("session"),
                        message.id,
                        MessageText::from("reply"),
                    )
                    .await?;
                assert_eq!(reply.id.0.len(), 40);
                assert_eq!(reply.user_id, UserId::from("session"));
                assert_eq!(&reply.text.0, "reply");
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn failed_spoke_if_not_exists_discussion() {
        try_execute(|db| {
            async move {
                db.new_discussion("title".to_string(), UserId::from("session"))
                    .await?;

                match db
                    .reply(
                        DiscussionId("ID".to_string()),
                        UserId::from("user2"),
                        MessageId("Null".to_string()),
                        MessageText::from("hello world!"),
                    )
                    .await
                {
                    Err(error::Error::DiscussionNotExists(id)) => {
                        assert_eq!(DiscussionId("ID".to_string()), id)
                    }
                    _ => panic!("expected DiscussionNotExists but was."),
                }
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn failed_spoke_if_not_exists_source_message() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;

                match db
                    .reply(
                        meta.id,
                        UserId::from("user2"),
                        MessageId("Null".to_string()),
                        MessageText::from("hello world!"),
                    )
                    .await
                {
                    Err(error::Error::MessageNotExists(id)) => {
                        assert_eq!(MessageId("Null".to_string()), id)
                    }
                    _ => panic!("expected DiscussionNotExists but was."),
                }
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn have_1_message() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;
                let message = db
                    .speak(
                        &meta.id,
                        UserId::from("user2"),
                        MessageText::from("hello world!"),
                    )
                    .await?;
                let discussion = db.discussion_by(&meta.id).await?;
                assert_eq!(discussion.meta, meta);
                assert_eq!(discussion.messages.len(), 1);
                assert_eq!(
                    discussion.messages[0],
                    MessageBundle {
                        message,
                        replies: Vec::with_capacity(0),
                    }
                );
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn have_1_reply() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("session"))
                    .await?;
                let message = db
                    .speak(
                        &meta.id,
                        UserId::from("user2"),
                        MessageText::from("hello world!"),
                    )
                    .await?;
                let reply = db
                    .reply(
                        meta.id.clone(),
                        UserId::from("user3"),
                        message.id.clone(),
                        MessageText::from("reply"),
                    )
                    .await?;
                let discussion = db.discussion_by(&meta.id).await?;
                assert_eq!(discussion.meta, meta);
                assert_eq!(discussion.messages.len(), 1);
                assert_eq!(discussion.messages[0].replies.len(), 1);
                assert_eq!(
                    discussion.messages[0],
                    MessageBundle {
                        message,
                        replies: vec![reply],
                    }
                );
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn have_3_replies() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("owner"))
                    .await?;
                let message = db
                    .speak(&meta.id, UserId::from("owner"), MessageText::from("1"))
                    .await?;
                let message2 = db
                    .speak(&meta.id, UserId::from("owner"), MessageText::from("2"))
                    .await?;
                let reply = db
                    .reply(
                        meta.id.clone(),
                        UserId::from("session"),
                        message2.id.clone(),
                        MessageText::from("reply1"),
                    )
                    .await?;
                let reply2 = db
                    .reply(
                        meta.id.clone(),
                        UserId::from("user2"),
                        message2.id.clone(),
                        MessageText::from("reply2"),
                    )
                    .await?;
                let discussion = db.discussion_by(&meta.id).await?;
                assert_eq!(discussion.meta, meta);
                assert_eq!(discussion.messages.len(), 2);
                assert_eq!(
                    discussion.messages[0],
                    MessageBundle {
                        message,
                        replies: vec![],
                    }
                );
                assert_eq!(
                    discussion.messages[1],
                    MessageBundle {
                        message: message2,
                        replies: vec![reply, reply2],
                    }
                );
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn close_discussion() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("owner"))
                    .await?;
                let message1 = db
                    .speak(&meta.id, UserId::from("owner"), MessageText::from("1"))
                    .await?;
                let message2 = db
                    .speak(&meta.id, UserId::from("owner"), MessageText::from("2"))
                    .await?;
                db.reply(
                    meta.id.clone(),
                    UserId::from("session"),
                    message2.id.clone(),
                    MessageText::from("reply1"),
                )
                .await?;
                db.reply(
                    meta.id.clone(),
                    UserId::from("user2"),
                    message2.id.clone(),
                    MessageText::from("reply2"),
                )
                .await?;
                db.close_discussion(&meta.id).await?;
                assert!(db.all_discussions().await?.is_empty());
                let mut con = db.lock();
                let tx = con.transaction()?;
                let messages = read_all_messages(&tx).unwrap();
                assert!(messages.is_empty());
                let message_ids = read_message_ids_in(&tx, meta.id)?;
                assert!(message_ids.is_empty());
                assert!(read_reply_message_ids_in(&tx, message1.id)?.is_empty());
                assert!(read_reply_message_ids_in(&tx, message2.id)?.is_empty());
                tx.commit()?;
                Ok(())
            }
        })
        .await;
    }

    #[tokio::test]
    async fn not_delete_second_discussion() {
        try_execute(|db| {
            async move {
                let meta = db
                    .new_discussion("title".to_string(), UserId::from("owner"))
                    .await?;

                db.speak(&meta.id, UserId::from("owner"), MessageText::from("1"))
                    .await?;
                let message2 = db
                    .speak(&meta.id, UserId::from("owner"), MessageText::from("2"))
                    .await?;
                db.reply(
                    meta.id.clone(),
                    UserId::from("session"),
                    message2.id.clone(),
                    MessageText::from("reply1"),
                )
                .await?;
                db.reply(
                    meta.id.clone(),
                    UserId::from("user2"),
                    message2.id.clone(),
                    MessageText::from("reply2"),
                )
                .await?;

                let meta2 = db
                    .new_discussion("title2".to_string(), UserId::from("owner"))
                    .await?;
                let message1 = db
                    .speak(
                        &meta2.id,
                        UserId::from("owner"),
                        MessageText::from("Discussion2 message1"),
                    )
                    .await?;
                let message2 = db
                    .speak(
                        &meta2.id,
                        UserId::from("owner"),
                        MessageText::from("Discussion2 message2"),
                    )
                    .await?;
                let reply1 = db
                    .reply(
                        meta2.id.clone(),
                        UserId::from("session"),
                        message2.id.clone(),
                        MessageText::from("Discussion2 reply1"),
                    )
                    .await?;
                let reply2 = db
                    .reply(
                        meta2.id,
                        UserId::from("user2"),
                        message2.id.clone(),
                        MessageText::from("Discussion2 reply2"),
                    )
                    .await?;

                db.close_discussion(&meta.id).await?;
                assert_eq!(db.all_discussions().await?.len(), 1);
                let mut con = db.lock();
                let tx = con.transaction()?;
                let messages = read_all_messages(&tx).unwrap();
                assert_eq!(messages, vec![message1, message2, reply1, reply2]);

                tx.commit()?;
                Ok(())
            }
        })
        .await;
    }

    async fn try_execute<F: Future<Output = error::Result>>(
        f: impl FnOnce(SqliteDiscussionIo) -> F + 'static + Send,
    ) {
        let room_id = RoomId::new();
        let path = format!("./{room_id}.db");
        let Ok(db) = SqliteDiscussionIo::new(room_id.clone()) else {
            if std::fs::metadata(&path).is_ok() {
                std::fs::remove_file(&path).unwrap();
            }
            return;
        };
        let result = std::panic::AssertUnwindSafe(f(db)).catch_unwind().await;
        delete_resource_dir(&room_id).unwrap();
        result.unwrap().unwrap();
    }
}
