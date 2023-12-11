use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::error;

pub mod thread;



#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum RequestCommand {
    Thread(thread::ThreadCommand),
}

impl Display for RequestCommand{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", serde_json::to_string(self).unwrap()))
    }
}



impl TryFrom<&str> for RequestCommand {
    type Error = error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
