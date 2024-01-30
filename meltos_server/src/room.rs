use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

use axum::response::Response;
use tokio::sync::Mutex;

use meltos::channel::{ChannelMessageSendable, MessageData, ResponseMessage};
use meltos::channel::request::RequestMessage;
use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_backend::path::room_resource_dir;
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_backend::sync::arc_mutex::ArcMutex;
use meltos_util::macros::Deref;

use crate::api::HttpResult;
use crate::error;

pub mod channel;


#[derive(Default, Clone, Deref)]
pub struct Rooms(ArcMutex<RoomMap>);


impl Rooms {
    pub async fn insert_room(&self, room: Room, life_time: Duration) {
        self.register_connect_timeout(room.id.clone());

        let rooms = self.0.clone();
        let room_id = room.id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(life_time).await;
            if let Err(e) = rooms.lock().await.delete(&room_id).await {
                tracing::error!("{e:?}");
            }
        });

        let mut rooms = self.0.lock().await;
        rooms.0.insert(room.id.clone(), room);
    }


    fn register_connect_timeout(&self, room_id: RoomId) {
        let rooms = self.0.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let mut rooms = rooms.lock().await;
            if rooms.room(&room_id).is_ok_and(|room| !room.is_connecting) {
                if let Err(e) = rooms.delete(&room_id).await {
                    tracing::error!("{e:?}");
                }
            }
        });
    }
}

#[derive(Default, Debug)]
pub struct RoomMap(HashMap<RoomId, Room>);

impl RoomMap {
    #[inline(always)]
    pub async fn delete(&mut self, room_id: &RoomId) -> HttpResult<()> {
        if let Some(room) = self.0.remove(room_id) {
            let result = room.send_all_users(ResponseMessage {
                from: room.owner.clone(),
                message: MessageData::ClosedRoom,
            })
                .await;
            room.delete_resource_dir();
            result?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn room(&mut self, room_id: &RoomId) -> std::result::Result<Room, Response> {
        Ok(self.room_mut(room_id)?.clone())
    }

    pub fn room_mut(&mut self, room_id: &RoomId) -> error::Result<&mut Room> {
        self.0.get_mut(room_id).ok_or_else(|| error::Error::RoomNotExists(room_id.clone()))
    }
}

#[derive(Clone)]
pub struct Room {
    pub owner: UserId,
    pub id: RoomId,
    pub session: Arc<dyn SessionIo>,
    capacity: u64,
    channels: Arc<Mutex<HashMap<UserId, Box<dyn ChannelMessageSendable<Error=error::Error>>>>>,
    is_connecting: bool,
}

impl Room {
    pub fn open<Session>(owner: UserId, capacity: u64) -> error::Result<Self>
        where
            Session: SessionIo + NewSessionIo + 'static,
    {
        let room_id = RoomId::default();

        Ok(Self {
            id: room_id.clone(),
            owner: owner.clone(),
            capacity,
            channels: Arc::new(Mutex::new(HashMap::new())),
            session: Arc::new(
                Session::new(room_id)
                    .map_err(|e| error::Error::FailedCreateSessionIo(e.to_string()))?,
            ),
            is_connecting: false,
        })
    }

    #[inline(always)]
    pub async fn error_if_reached_capacity(&self) -> error::Result {
        let current_user_count = self.session.user_count().await?;
        if self.capacity <= current_user_count {
            Err(error::Error::ReachedCapacity(self.capacity))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn set_connecting(&mut self) {
        self.is_connecting = true;
    }

    pub async fn insert_channel(
        &self,
        channel: impl ChannelMessageSendable<Error=error::Error> + 'static,
    ) {
        let mut channels = self.channels.lock().await;
        channels.insert(channel.user_id().clone(), Box::new(channel));
    }

    #[inline(always)]
    pub async fn send_request(&self, message: RequestMessage) -> HttpResult<()> {
        let mut channels = self.channels.lock().await;
        let channel = channels.get_mut(&self.owner).ok_or_else(|| crate::error::Error::RoomOwnerDisconnected(self.id.clone()))?;
        if let Err(_) = channel.send_request(message).await {
            drop(channels);
            self
                .send_all_users(ResponseMessage {
                    from: self.owner.clone(),
                    message: MessageData::ClosedRoom,
                })
                .await?;
        }
        Ok(())
    }

    #[inline(always)]
    pub async fn send_to(&self, to: &UserId, message: ResponseMessage) -> HttpResult<()> {
        let mut channels = self.channels.lock().await;
        let channel = channels.get_mut(to).ok_or_else(|| crate::error::Error::UserNotExists(self.id.clone(), to.clone()))?;
        if let Err(_) = channel.send_response(message).await {
            channels.remove(to);
        }
        Ok(())
    }

    #[inline(always)]
    pub async fn send_all_users(
        &self,
        message: ResponseMessage,
    ) -> std::result::Result<(), Response> {
        let mut channels = self.channels.lock().await;
        let mut next_channels = HashMap::with_capacity(channels.len());
        for (user_id, mut sender) in channels.into_iter() {
            if let Err(e) = sender.send_response(message.clone()).await {
                // 失敗した場合は切断されたと判断し、ログだけ出力してchannelsから消す
                tracing::debug!("{e}");
            } else {
                next_channels.insert(user_id, sender);
            }
        }
        *channels = next_channels;
        Ok(())
    }

    pub async fn leave(&self, user_id: UserId) -> error::Result {
        self.session.unregister(user_id.clone()).await?;
        Ok(())
    }

    pub fn delete_resource_dir(self) {
        drop(self.session);

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
}

impl Debug for Room {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("id", &self.id)
            .field("owner", &self.owner)
            .finish()
    }
}
