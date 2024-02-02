use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

use axum::response::Response;
use tokio::sync::Mutex;

use meltos::channel::{ChannelMessageSendable, MessageData, ResponseMessage};
use meltos::channel::request::UserRequest;
use meltos::room::RoomId;
use meltos::user::{SessionId, UserId};
use meltos_backend::path::room_resource_dir;
use meltos_backend::session::{NewSessionIo, SessionIo};
use meltos_backend::sync::arc_mutex::ArcMutex;
use meltos_util::macros::Deref;

use crate::api::HttpResult;
use crate::error;
use crate::room::channel::{Channels, WebsocketSender};

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
            tracing::info!("TIMEOUT LIFE CYCLE: Room {room_id} was deleted");
        });

        let mut rooms = self.0.lock().await;
        rooms.0.insert(room.id.clone(), room);
    }


    pub async fn delete(&self, room_id: &RoomId) -> HttpResult<()> {
        let mut map = self.lock().await;
        map.delete(room_id).await?;
        Ok(())
    }


    fn register_connect_timeout(&self, room_id: RoomId) {
        let rooms = self.0.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let mut rooms = rooms.lock().await;
            let Ok(room) = rooms.room(&room_id) else {
                return;
            };
            if !room.is_connecting_owner().await {
                drop(room);
                tracing::debug!("room owner timeout connected channel room_id: {room_id}");
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
    wait_users: Arc<Mutex<HashSet<UserId>>>,
    capacity: u64,
    channels: Channels,
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
            channels: Channels::default(),
            session: Arc::new(
                Session::new(room_id)
                    .map_err(|e| error::Error::FailedCreateSessionIo(e.to_string()))?,
            ),
            wait_users: Arc::new(Mutex::new(HashSet::new())),
        })
    }


    pub async fn join(&self, user_id: Option<UserId>) -> error::Result<(UserId, SessionId)> {
        let (user_id, session_id) = self.session.register(user_id).await?;
        let wait_users = self.wait_users.clone();
        let session = self.session.clone();
        let user_id_cloned = user_id.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let mut users = wait_users.lock().await;
            if !users.remove(&user_id_cloned) {
                if let Err(e) = session.unregister(user_id_cloned).await {
                    tracing::error!("{e}");
                }
            }
        });

        Ok((user_id, session_id))
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
    pub async fn is_connecting_owner(&self) -> bool {
        let owner_id = &self.owner;
        self.wait_users.lock().await.remove(owner_id)
    }

    #[inline(always)]
    pub async fn set_connecting(&self, user_id: UserId) {
        self.wait_users.lock().await.insert(user_id);
    }

    pub async fn insert_channel(
        &self,
        channel: WebsocketSender,
    ) {
        self.channels.insert_channel(channel).await
    }

    #[inline(always)]
    pub async fn send_request(&self, request: UserRequest) -> HttpResult<()> {
        let mut channels = self.channels.0.lock().await;
        tracing::debug!("{:?}", channels.keys());

        let channel = channels.get_mut(&self.owner).ok_or_else(|| {
            tracing::error!("NOT FOUND!");
            crate::error::Error::RoomOwnerDisconnected(self.id.clone())
        })?;
        if channel.send_request(request).await.is_err() {
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
    pub async fn send_all_users(
        &self,
        message: ResponseMessage,
    ) -> std::result::Result<(), Response> {
        // let mut channels = self.channels.lock().await;
        // let mut next_channels: ChannelMap = HashMap::with_capacity(channels.len());
        // let keys: Vec<UserId> = channels.keys().cloned().collect();
        // for user_id in keys {
        //     let Some(mut sender) = next_channels.remove(&user_id) else { continue; };
        //     if let Err(e) = sender.send_response(message.clone()).await {
        //         // 失敗した場合は切断されたと判断し、ログだけ出力してchannelsから消す
        //         tracing::debug!("{e}");
        //     } else {
        //         next_channels.insert(user_id, sender);
        //     }
        // }
        // *channels = next_channels;
        Ok(())
    }

    pub async fn leave(&self, user_id: UserId) -> error::Result {
        self.session.unregister(user_id.clone()).await?;
        Ok(())
    }

    pub fn delete_resource_dir(self) {
        let dir = room_resource_dir(&self.id);
        let room_id = self.id;
        drop(self.session);

        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir) {
                log::error!(
                    "failed delete room resource dir \nroom_id : {} \nmessage: {e}",
                    room_id
                );
            } else {
                tracing::debug!("removed database; room_id: {room_id}");
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
