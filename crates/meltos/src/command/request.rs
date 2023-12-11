use serde::{Deserialize, Serialize};
use crate::error;

pub mod thread;



#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum RequestCommand {
    Thread(thread::ThreadCommand),
}


impl TryFrom<&[u8]> for RequestCommand {
    type Error = error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(bincode::deserialize(value)?)
    }
}
