use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use meltos_util::impl_string_new_type;

use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
#[wasm_bindgen(getter_with_clone)]
pub struct BranchName(pub String);
impl_string_new_type!(BranchName);

impl BranchName {
    #[inline]
    pub fn main() -> Self {
        Self::from("main")
    }

    pub fn working<Fs>(fs: Fs) -> std::io::Result<Self>
    where
        Fs: FileSystem,
    {
        WorkingIo::new(fs).try_read()
    }
}
