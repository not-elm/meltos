use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::response::Response;
use serde::Serialize;
use tokio::sync::Mutex;

use meltos::channel::{ChannelMessage, ChannelMessageSendable};
use meltos::discussion::DiscussionBundle;
use meltos::room::RoomId;
use meltos::schema::room::RoomBundle;
use meltos::user::UserId;
use meltos_backend::discussion::{DiscussionIo, NewDiscussIo};
use meltos_backend::path::{create_resource_dir, room_resource_dir};
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_backend::sync::arc_mutex::ArcMutex;
use meltos_backend::tvc::TvcBackendIo;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::io::bundle::Bundle;
use meltos_util::macros::Deref;

use crate::api::HttpResult;
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

    pub fn room_mut(&mut self, room_id: &RoomId) -> error::Result<&mut Room> {
        self.0.get_mut(room_id).ok_or(error::Error::RoomNotExists)
    }
}

#[derive(Clone)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    pub tvc: TvcBackendIo<StdFileSystem>,
    pub session: Arc<dyn SessionIo>,
    capacity: u64,
    discussion: Arc<dyn DiscussionIo>,
    channels: Arc<Mutex<Vec<Box<dyn ChannelMessageSendable<Error=error::Error>>>>>,
}

impl Room {
    pub fn open<Discussion, Session>(owner: UserId, capacity: u64) -> error::Result<Self>
        where
            Discussion: DiscussionIo + NewDiscussIo + 'static,
            Session: SessionIo + NewSessionIo + 'static,
    {
        let room_id = RoomId::default();
        create_resource_dir(&room_id)?;
        Ok(Self {
            id: room_id.clone(),
            owner: owner.clone(),
            capacity,
            discussion: Arc::new(
                Discussion::new(room_id.clone())
                    .map_err(|e| error::Error::FailedCreateDiscussionIo(e.to_string()))?,
            ),
            tvc: TvcBackendIo::new(room_id.clone(), StdFileSystem),
            channels: Arc::new(Mutex::new(Vec::new())),
            session: Arc::new(
                Session::new(room_id)
                    .map_err(|e| error::Error::FailedCreateSessionIo(e.to_string()))?,
            ),
        })
    }

    #[inline(always)]
    pub async fn error_if_reached_capacity(&self) -> error::Result{
        let current_user_count = self.session.user_count().await?;
        if self.capacity <= current_user_count{
            Err(error::Error::ReachedCapacity(self.capacity))
        }else{
            Ok(())
        }
    }


    pub async fn room_bundle(&self) -> error::Result<RoomBundle> {
        let discussion = self
            .discussion
            .all_discussions()
            .await?;
        let tvc = self.tvc.bundle().await?;

        Ok(RoomBundle {
            tvc,
            discussion,
        })
    }

    pub async fn insert_channel(
        &self,
        channel: impl ChannelMessageSendable<Error=error::Error> + 'static,
    ) {
        let mut channels = self.channels.lock().await;
        channels.push(Box::new(channel));
    }

    #[inline(always)]
    pub async fn send_all_users(
        &self,
        message: ChannelMessage,
    ) -> std::result::Result<(), Response> {
        let mut channels = self.channels.lock().await;
        let mut next_channels = Vec::with_capacity(channels.len());
        while let Some(mut sender) = channels.pop() {
            if let Err(e) = sender.send(message.clone()).await {
                // 失敗した場合は切断されたと判断し、ログだけ出力してchannelsから消す
                tracing::debug!("{e}");
            } else {
                next_channels.push(sender);
            }
        }
        *channels = next_channels;
        Ok(())
    }

    #[inline(always)]
    pub async fn tvc_repository_size(&self) -> error::Result<usize> {
        self.tvc.total_objs_size().await.map_err(crate::error::Error::Tvc)
    }

    #[inline(always)]
    pub async fn save_bundle(&self, bundle: Bundle) -> error::Result {
        self.tvc.save(bundle).await?;
        Ok(())
    }

    pub async fn discussions(&self) -> HttpResult<Vec<DiscussionBundle>> {
        let discussions = self.discussion.all_discussions().await?;
        Ok(discussions)
    }

    #[inline(always)]
    pub async fn create_bundle(&self) -> error::Result<Bundle> {
        self.tvc.bundle().await.map_err(crate::error::Error::Tvc)
    }

    #[inline(always)]
    pub async fn write_head(&self, user_id: UserId) -> error::Result {
        self.tvc.write_head(&BranchName(user_id.0)).await.map_err(crate::error::Error::Tvc)
    }


    pub async fn leave(&self, user_id: UserId) -> error::Result{
        self.session.unregister(user_id.clone()).await?;
        self.tvc.leave(user_id).await?;
        Ok(())
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

    pub fn delete_resource_dir(self) {
        drop(self.session);
        drop(self.discussion);
        let dir = room_resource_dir(&self.id);
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                log::error!(
                    "failed delete room resource dir \nroom_id : {} \nmessage: {e}",
                    self.id
                );
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
