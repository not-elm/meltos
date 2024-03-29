use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::error;
use meltos_util::impl_string_new_type;

use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;

#[wasm_bindgen(getter_with_clone)]
#[repr(transparent)]
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct BranchName(pub String);
impl_string_new_type!(BranchName);

impl BranchName {
    #[inline]
    pub fn owner() -> Self {
        Self::from("owner")
    }

    pub async fn working<Fs>(fs: Fs) -> error::Result<Self>
    where
        Fs: FileSystem,
    {
        WorkingIo::new(fs).try_read().await
    }
}
