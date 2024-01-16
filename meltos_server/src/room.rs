use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Serialize;
use serde_json::json;
use tokio::sync::Mutex;

use meltos::channel::{ChannelMessage, ChannelMessageSendable};
use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
use meltos_backend::path::{create_resource_dir, room_resource_dir};
use meltos_backend::sync::arc_mutex::ArcMutex;
use meltos_backend::tvc::TvcBackendIo;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::io::bundle::Bundle;
use meltos_util::macros::Deref;

use crate::error;
use crate::room::executor::discussion::DiscussionCommandExecutor;

pub mod channel;
mod executor;

#[derive(Default, Clone, Debug, Deref)]
pub struct Rooms(ArcMutex<RoomMap>);

impl Rooms {
    pub async fn insert_room(&self, room: Room, life_time: Duration) {
        let rooms = self.0.clone();
        let room_id = room.id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(life_time).await;
            rooms.lock().await.delete(&room_id);
        });

        let mut rooms = self.0.lock().await;
        rooms.0.insert(room.id.clone(), room);
    }
}

#[derive(Default, Debug)]
pub struct RoomMap(HashMap<RoomId, Room>);

impl RoomMap {
    #[inline(always)]
    pub fn delete(&mut self, room_id: &RoomId) {
        if let Some(room) = self.0.remove(room_id) {
            room.delete_resource_dir();
        }
    }

    #[inline(always)]
    pub fn room(&mut self, room_id: &RoomId) -> std::result::Result<Room, Response> {
        Ok(self.room_mut(room_id)?.clone())
    }

    pub fn room_mut(&mut self, room_id: &RoomId) -> std::result::Result<&mut Room, Response> {
        self.0.get_mut(room_id).ok_or(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(
                    json!({
                        "error": format!("room_id {room_id} is not exists")
                    })
                        .to_string(),
                ))
                .unwrap(),
        )
    }
}

#[derive(Clone)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    pub tvc: TvcBackendIo<StdFileSystem>,
    discussion: Arc<dyn DiscussionIo>,
    channels: Arc<Mutex<Vec<Box<dyn ChannelMessageSendable<Error=error::Error>>>>>,
}

impl Room {
    pub fn open<Discussion: DiscussionIo + NewDiscussIo + 'static>(owner: UserId) -> error::Result<Self> {
        let room_id = RoomId::default();
        create_resource_dir(&room_id)?;
        Ok(Self {
            id: room_id.clone(),
            owner: owner.clone(),
            discussion: Arc::new(Discussion::new(room_id.clone()).map_err(|e| error::Error::RoomCreateFailed(e.to_string()))?),
            tvc: TvcBackendIo::new(room_id, StdFileSystem),
            channels: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn insert_channel(
        &self,
        channel: impl ChannelMessageSendable<Error=error::Error> + 'static,
    ) {
        let mut channels = self.channels.lock().await;
        channels.push(Box::new(channel));
    }

    pub async fn send_all_users(
        &self,
        message: ChannelMessage,
    ) -> std::result::Result<(), Response> {
        let mut channels = self.channels.lock().await;
        for sender in channels.iter_mut() {
            sender.send(message.clone()).await?;
        }
        Ok(())
    }

    pub fn save_bundle(&self, bundle: Bundle) -> std::result::Result<(), Response> {
        self.tvc.save(bundle).map_err(|e| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(
                    json!({
                        "error" : e.to_string()
                    })
                        .to_string(),
                ))
                .unwrap()
        })?;

        Ok(())
    }

    pub fn create_bundle(&self) -> std::result::Result<Bundle, Response> {
        match self.tvc.bundle() {
            Ok(bundle) => Ok(bundle),
            Err(error) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(
                        json!({
                            "error": error.to_string()
                        })
                            .to_string(),
                    ))
                    .unwrap();
                Err(response)
            }
        }
    }

    pub async fn global_discussion<'a, F, O, S>(&'a self, user_id: UserId, f: F) -> error::Result<S>
        where
            F: FnOnce(DiscussionCommandExecutor<'a, dyn DiscussionIo>) -> O,
            O: Future<Output=error::Result<S>>,
            S: Serialize,
    {
        let command = f(self.as_global_discussion_executor(user_id)).await?;
        Ok(command)
    }

    pub fn delete_resource_dir(&self) {
        let dir = room_resource_dir(&self.id);
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                log::error!("failed delete room resource dir \nroom_id : {} \nmessage: {e}", self.id);
            }
        }
    }

    fn as_global_discussion_executor(
        &self,
        user_id: UserId,
    ) -> DiscussionCommandExecutor<'_, dyn DiscussionIo> {
        DiscussionCommandExecutor::new(user_id, self.discussion.as_ref())
    }
}

impl Debug for Room {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .finish()
    }
}
