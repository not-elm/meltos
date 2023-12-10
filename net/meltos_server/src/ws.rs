use axum::extract::ws::Message;

use meltos::order::ServerOrder;

use crate::error;

pub mod receiver;
pub mod repository;
pub mod sender;


pub trait AsOrder {
    /// Convert to [`ServerOrder`] from `Self`
    fn as_order(&self) -> error::Result<ServerOrder>;
}


impl AsOrder for Message {
    fn as_order(&self) -> error::Result<ServerOrder> {
        match self {
            Message::Binary(binary) => Ok(ServerOrder::try_from(binary.as_slice())?),
            _ => Err(error::Error::InvalidOrder),
        }
    }
}
