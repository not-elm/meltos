use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use meltos_util::macros::{Display, Sha1};

#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Sha1)]
pub struct UserId(String);
impl_string_new_type!(UserId);

#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display, Sha1)]
pub struct SessionId(pub String);

