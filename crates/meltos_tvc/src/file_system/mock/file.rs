use wasm_bindgen::prelude::wasm_bindgen;

use crate::file_system::{Stat, StatType};
use crate::time::since_epoch_secs;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MockFile {
    pub create_time: u64,
    pub update_time: u64,
    pub buf: Vec<u8>,
}


impl MockFile {
    #[inline]
    pub fn new(buf: Vec<u8>) -> Self {
        let create_time = since_epoch_secs();
        Self {
            create_time,
            update_time: create_time,
            buf,
        }
    }


    #[inline(always)]
    pub fn stat(&self) -> Stat {
        Stat {
            create_time: self.create_time,
            update_time: self.update_time,
            ty: StatType::File,
            size: self.buf.len() as u64,
        }
    }
}