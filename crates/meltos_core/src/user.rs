use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::impl_string_new_type;

#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct UserId(String);
impl_string_new_type!(UserId);
