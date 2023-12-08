use meltos_core::impl_string_new_type;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct BranchName(String);
impl_string_new_type!(BranchName);
