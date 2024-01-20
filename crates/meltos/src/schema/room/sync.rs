use serde::{Deserialize, Serialize};
use meltos_tvc::io::bundle::Bundle;
use crate::discussion::DiscussionBundle;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomBundle{
    pub tvc: Bundle,
    pub discussion: Vec<DiscussionBundle>,
}