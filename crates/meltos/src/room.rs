use serde::{Deserialize, Serialize};

use meltos_util::macros::{Display, Sha1};

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize, Display, Sha1)]
pub struct RoomId(pub String);


