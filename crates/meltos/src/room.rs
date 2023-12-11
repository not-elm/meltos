use serde::{Deserialize, Serialize};

use meltos_util::macros::Display;


#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize, Display)]
pub struct RoomId(pub String);


impl Default for RoomId {
    fn default() -> Self {
        RoomId(uuid::Uuid::new_v4().to_string())
    }
}
