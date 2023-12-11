pub mod thread;

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use crate::error;

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub enum ClientCommand {
    Thread(thread::ThreadOrder)
}


impl TryFrom<Message> for ClientCommand{
    type Error = error::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => {
                Ok(serde_json::from_str(&text)?)
            },
            _ => Err(error::Error::SerializeClientCommand)
        }
    }
}