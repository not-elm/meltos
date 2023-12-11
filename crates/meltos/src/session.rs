use serde::{Deserialize, Serialize};

use meltos_util::macros::Display;

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Display)]
pub struct RoomId(pub String);


