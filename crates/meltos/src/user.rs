use meltos_util::impl_string_new_type;
use meltos_util::macros::Display;
use serde::{Deserialize, Serialize};


#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct UserId(String);
impl_string_new_type!(UserId);


#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash, Display)]
pub struct SessionId(pub String);
