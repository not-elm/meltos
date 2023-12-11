use axum::extract::ws::WebSocket;
use futures::stream::SplitStream;
use futures::StreamExt;

use meltos::command::request::RequestCmd;

use crate::error;
use crate::room::ws::AsRequestCommand;

pub struct CommandReceiver(pub(crate) SplitStream<WebSocket>);


impl CommandReceiver {
    pub async fn recv(&mut self) -> error::Result<RequestCmd> {
        let message = self.0.next().await.ok_or(error::Error::Disconnected)??;
        message.as_request_command()
    }
}
