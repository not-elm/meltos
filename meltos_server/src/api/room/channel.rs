use std::convert::Infallible;
use std::time::Duration;
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::Message;
use axum::response::{Response, Sse};
use axum::response::sse::Event;
use futures::{Stream, stream, StreamExt};
use tokio::io::BufStream;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_tungstenite::tungstenite::handshake::headers;

use crate::middleware::room::SessionRoom;
use crate::middleware::user::SessionUser;
use crate::room::channel::WebsocketSender;
use crate::room::Rooms;

#[tracing::instrument(skip(rooms))]
pub async fn channel(
    State(rooms): State<Rooms>,
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
            while let Some(Ok(message)) = rx.next().await {
                if matches!(message, Message::Close(_)) {
                    break;
                }
            }
            if room.owner == user_id {
                if let Err(e) = rooms.delete(&room.id).await {
                    tracing::error!("{e:?}");
                }
            } else if let Err(e) = room.leave(user_id).await {
                tracing::error!("{e}");
            }
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
