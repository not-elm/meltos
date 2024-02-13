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
    room.set_connecting(user_id.clone()).await;

    ws.on_upgrade(|socket| {
        async move {
            let (tx, mut rx) = socket.split();
            let sender = WebsocketSender::new(user_id.clone(), tx);
            room.insert_channel(sender).await;
        }
    })
}

// async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
//
//
//     let stream = BroadcastStream::new();
//
//     Sse::new(stream).keep_alive(
//         axum::response::sse::KeepAlive::new()
//             .interval(Duration::from_secs(1))
//             .text("keep-alive-text"),
//     )
// }
