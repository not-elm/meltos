use crate::discussion::structs::id::DiscussionId;
use crate::discussion::structs::message::MessageNo;
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
    ThreadNotExists(DiscussionId),

    #[error("failed reply message no {0:?} is not exists")]
    MessageNoNotExists(MessageNo),

    #[error("websocket message can't serialize to client command")]
    SerializeClientCommand,
}
