use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use futures::stream::{SplitSink, SplitStream};
use futures::StreamExt;
use meltos::session::RoomId;
use serde::{Deserialize, Serialize};

use meltos_util::error::LogIfError;

use crate::api::webrtc::{BroadcastReceiver, SocketChannels};
use crate::error;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub session_id: RoomId,
    capacity: Option<usize>,
}


impl Param {
    pub fn capacity(&self) -> usize {
        self.capacity.unwrap_or(30).max(50)
    }
}


pub async fn connect(
    ws: WebSocketUpgrade,
    Query(param): Query<Param>,
    State(channels): State<SocketChannels>,
) -> Response {
    let (tx, rx) = tokio::sync::broadcast::channel(param.capacity());
    channels.lock().await.insert(param.session_id, tx);
    ws.on_upgrade(move |socket| websocket_handle(socket, rx))
}


async fn websocket_handle(socket: WebSocket, broadcast_rx: BroadcastReceiver) {
    let (ws_tx, ws_rx) = socket.split();
    let h1 = receive_offer_message(ws_rx);
    let h2 = receive_answers_message(ws_tx, broadcast_rx);
    let (r1, r2) = tokio::join!(h1, h2);
    r1.log_if_error();
    r2.log_if_error();
}


async fn receive_offer_message(mut ws_rx: SplitStream<WebSocket>) -> error::Result {
    tokio::spawn(async move { while let Some(Ok(_message)) = ws_rx.next().await {} }).await?;

    Ok(())
}


async fn receive_answers_message(
    mut _ws_tx: SplitSink<WebSocket, Message>,
    mut broadcast_rx: BroadcastReceiver,
) -> error::Result {
    tokio::spawn(async move { while let Ok(_message) = broadcast_rx.recv().await {} }).await?;

    Ok(())
}
