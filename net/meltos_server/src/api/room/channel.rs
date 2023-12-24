use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use futures::StreamExt;

use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;
use crate::room::channel::WebsocketSender;

#[tracing::instrument]
pub async fn channel(
    ws: WebSocketUpgrade,
    SessionRoom(room): SessionRoom,
    SessionUser(user_id): SessionUser,
) -> Response {
    ws.on_upgrade(|socket| async move {
        let (tx, _) = socket.split();
        let sender = WebsocketSender::new(user_id.clone(), tx);
        room.insert_channel(sender).await;
    })
}

