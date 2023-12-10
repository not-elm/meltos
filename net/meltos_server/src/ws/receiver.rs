use axum::extract::ws::WebSocket;
use futures::stream::SplitStream;
use futures::StreamExt;

use meltos::order::ServerOrder;

use crate::error;
use crate::ws::AsOrder;

pub struct WebsocketReceiver(pub(crate) SplitStream<WebSocket>);


impl WebsocketReceiver {
    pub async fn recv(&mut self) -> error::Result<ServerOrder> {
        let message = self.0.next().await.ok_or(error::Error::Disconnected)??;

        message.as_order()
    }
}
