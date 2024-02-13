use std::collections::HashMap;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use futures::stream::SplitSink;
use serde_json::json;
use tokio::sync::Mutex;

use meltos::channel::{ChannelMessageSendable, MessageData, ResponseMessage};
use meltos::channel::request::UserRequest;
use meltos::room::RoomId;
use meltos::user::UserId;

use crate::error;

type ChannelMap = HashMap<UserId, WebsocketSender>;


#[derive(Default)]
pub struct Channels(Arc<Mutex<ChannelMap>>);


impl Channels {
    pub async fn send_request(
        &self,
        owner: UserId,
        room_id: RoomId,
        request: UserRequest,
    ) -> error::Result {
        let mut channels = self.0.lock().await;
        tracing::debug!("{:?}", channels.keys());

        let channel = channels.get_mut(&owner).ok_or_else(|| {
            error::Error::RoomOwnerDisconnected(room_id)
        })?;

        if channel.send_request(request).await.is_err() {
            drop(channels);
            self.response_all(&ResponseMessage {
                from: owner,
                message: MessageData::ClosedRoom,
            })
                .await;
        }
        Ok(())
    }


    pub async fn response_all(&self, message: &ResponseMessage) {
        let all_users = self.0.lock().await.keys().cloned().collect::<Vec<UserId>>();
        self.response(&all_users, message).await
    }


    pub async fn response(
        &self,
        to: &[UserId],
        message: &ResponseMessage,
    ) {
        let mut channels = self.0.lock().await;
        let mut delete_users = Vec::with_capacity(channels.len());

        for (user_id, channel) in channels
            .iter_mut()
            .filter(|(id, _)| to.contains(id))
        {
            if let Err(_) = channel.send_response(message).await {
                delete_users.push(user_id.clone());
            }
        }

        for user in delete_users.iter() {
            channels.remove(user);
        }
    }


    pub async fn insert_channel(
        &self,
        channel: WebsocketSender,
    ) {
        let mut channels = self.0.lock().await;
        tracing::info!("insert1 {:?}", channels.keys());
        channels.insert(channel.user_id().clone(), channel);
        tracing::info!("insert2 {:?}", channels.keys());
    }
}

impl Clone for Channels {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


pub struct WebsocketSender {
    pub user_id: UserId,
    pub tx: SplitSink<WebSocket, Message>,
}

impl WebsocketSender {
    #[inline]
    pub const fn new(user_id: UserId, tx: SplitSink<WebSocket, Message>) -> WebsocketSender {
        Self {
            user_id,
            tx,
        }
    }
}

#[async_trait]
impl ChannelMessageSendable for WebsocketSender {
    type Error = crate::error::Error;

    #[inline(always)]
    fn user_id(&self) -> &UserId {
        &self.user_id
    }

    async fn send_request(&mut self, request: UserRequest) -> Result<(), Self::Error> {
        let req = serde_json::to_string(&request).unwrap();
        self.tx
            .send(Message::Text(json!({
                "type" : "request",
                "data" : req
            }).to_string()))
            .await
            .map_err(|e| crate::error::Error::FailedSendChannelMessage(e.to_string()))?;
        Ok(())
    }

    async fn send_response(&mut self, message: &ResponseMessage) -> crate::error::Result {
        self.tx
            .send(Message::Text(serde_json::to_string(message)?))
            .await
            .map_err(|e| crate::error::Error::FailedSendChannelMessage(e.to_string()))?;
        Ok(())
    }
}
