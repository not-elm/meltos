use crate::thread::structs::id::ThreadId;
use crate::thread::structs::message::MessageNo;
use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    StdBoxed(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    BinCode(#[from] bincode::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("structs not exists id = {0:?}")]
    ThreadNotExists(ThreadId),

    #[error("failed reply message no {0:?} is not exists")]
    MessageNoNotExists(MessageNo),

    #[error("websocket message can't serialize to client command")]
    SerializeClientCommand,
}
