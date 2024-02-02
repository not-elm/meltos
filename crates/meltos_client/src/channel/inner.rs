use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

use meltos::room::RoomId;

#[derive(Debug)]
pub struct RoomInnerChannel {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}


impl RoomInnerChannel {
    pub async fn connect(room_id: &RoomId) -> crate::error::Result<Self> {
        let (stream, _) = connect_async(format!("ws://localhost:3000/room/{room_id}/channel"))
            .await
            .unwrap();

        Ok(Self {
            stream
        })
    }

    pub async fn next(&mut self) -> Option<String> {
        loop {
            let Ok(message) = self.stream.next().await? else {
                return None;
            };
            
            let Message::Text(text) = message else {
                continue;
            };

            return Some(text);
        }
    }
}