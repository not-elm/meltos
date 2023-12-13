use serde::{Deserialize, Serialize};

use meltos_util::macros::{Display, Sha1};

#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Hash, Debug, Deserialize, Serialize, Display, Sha1)]
pub struct DiscussionId(pub String);

