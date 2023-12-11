use axum::extract::ws::WebSocket;
use futures::stream::SplitStream;
use futures::StreamExt;

use meltos::command::request::RequestCommand;

use crate::error;
use crate::room::ws::AsOrder;

pub struct CommandReceiver(pub(crate) SplitStream<WebSocket>);


impl CommandReceiver {
    pub async fn recv(&mut self) -> error::Result<RequestCommand> {
        let message = self.0.next().await.ok_or(error::Error::Disconnected)??;
        message.as_order()
    }
}
