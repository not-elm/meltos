use crate::error;
use axum::extract::ws::WebSocket;
use futures::stream::SplitSink;
use futures::SinkExt;
use meltos::order::ServerOrder;
use meltos::session::SessionId;
use meltos::user::UserId;
use meltos_util::serde::AsBinary;


#[derive(Debug)]
pub struct CommandSender {
    pub(crate) session_id: SessionId,
    pub(crate) user_id: UserId,
    pub(crate) sender: SplitSink<WebSocket, axum::extract::ws::Message>,
}


impl CommandSender {
    #[inline]
    pub fn session_with_in(&self, session_id: &SessionId) -> bool {
        &self.session_id == session_id
    }


    #[inline]
    pub fn is_target(&self, session_id: &SessionId, user_id: &UserId) -> bool {
        &self.session_id == session_id && &self.user_id == user_id
    }


    pub async fn send_order(&mut self, order: &ServerOrder) -> error::Result {
        let binary = order
            .as_binary()
            .map_err(|_| error::Error::SerializeToBinary)?;
        if let Err(error) = self
            .sender
            .send(axum::extract::ws::Message::Binary(binary))
            .await
        {
            Err(error::Error::from(error))
        } else {
            Ok(())
        }
    }
}
