use axum::extract::ws::Message;

use meltos::command::request::RequestCommand;

use crate::error;

pub mod receiver;


pub trait AsOrder {
    /// Convert to [`ServerOrder`] from `Self`
    fn as_order(&self) -> error::Result<RequestCommand>;
}


impl AsOrder for Message {
    fn as_order(&self) -> error::Result<RequestCommand> {
        match self {
            Message::Binary(binary) => Ok(RequestCommand::try_from(binary.as_slice())?),
            _ => Err(error::Error::InvalidOrder),
        }
    }
}
