pub mod client;
pub mod thread;

use crate::error;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct ServerOrder {
    pub session_id: SessionId,
    pub command: Command,
}


#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum Command {
    Thread(thread::ThreadCommand),
}


impl TryFrom<&[u8]> for ServerOrder {
    type Error = error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(bincode::deserialize(value)?)
    }
}
