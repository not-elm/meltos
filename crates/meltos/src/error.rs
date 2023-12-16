use thiserror::Error;

use crate::discussion::id::DiscussionId;
use crate::discussion::message::MessageId;

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

    #[error("discussion not exists id = {0}")]
    DiscussionNotExists(DiscussionId),

    #[error("message not exists id = {0}")]
    MessageNotExists(MessageId),

    #[error("websocket message can't serialize to remote_client command")]
    SerializeClientCommand,
}
