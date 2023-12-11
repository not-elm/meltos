use serde::{Deserialize, Serialize};

use crate::command::request::RequestCmd;
use crate::user::UserId;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct ServerCommand {
    pub from: UserId,
    pub command: RequestCmd,
}
