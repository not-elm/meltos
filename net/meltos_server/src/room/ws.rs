use axum::extract::ws::Message;

use meltos::command::request::RequestCmd;

use crate::error;

pub mod receiver;


pub trait AsRequestCommand {
    /// Convert to [`ServerOrder`] from `Self`
    fn as_request_command(&self) -> error::Result<RequestCmd>;
}


impl AsRequestCommand for Message {
    fn as_request_command(&self) -> error::Result<RequestCmd> {
        match self {
            Message::Text(text) => Ok(RequestCmd::try_from(text.as_str())?),
            _ => Err(error::Error::InvalidOrder),
        }
    }
}
