use thiserror::Error;

use crate::structs::id::ThreadId;
use crate::structs::message::MessageNo;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("structs not exists id = {0:?}")]
    ThreadNotExists(ThreadId),

    #[error("failed reply message no {0:?} is not exists")]
    MessageNoNotExists(MessageNo),
}
