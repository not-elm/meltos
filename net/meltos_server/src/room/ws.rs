use axum::extract::ws::Message;

use meltos::command::request::RequestCommand;

use crate::error;

pub mod receiver;


pub trait AsRequestCommand {
    /// Convert to [`ServerOrder`] from `Self`
    fn as_request_command(&self) -> error::Result<RequestCommand>;
}


impl AsRequestCommand for Message {
    fn as_request_command(&self) -> error::Result<RequestCommand> {
        match self {
            Message::Text(text) => Ok(RequestCommand::try_from(text.as_str())?),
            _ => Err(error::Error::InvalidOrder),
        }
    }
}
