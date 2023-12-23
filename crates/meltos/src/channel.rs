use crate::schema::room::Joined;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChannelMessage {
    Joined(Joined),
}
