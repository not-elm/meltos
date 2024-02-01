use std::sync::{Arc, RwLock};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::file_system::{Stat, StatType};
use crate::time::since_epoch_secs;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct MemoryFile(Arc<RwLock<MemoryFileInner>>);

impl MemoryFile {
    #[inline(always)]
    pub fn new(buf: Vec<u8>) -> Self {
        Self(Arc::new(RwLock::new(MemoryFileInner::new(buf))))
    }

    #[inline(always)]
    pub fn stat(&self) -> Stat {
        self.0.read().unwrap().stat()
    }

    #[inline(always)]
    pub fn buf(&self) -> Vec<u8> {
        self.0.read().unwrap().buf.clone()
    }

    #[inline(always)]
    pub fn write(&self, buf: Vec<u8>) {
        self.0.write().unwrap().buf = buf;
    }

    #[inline(always)]
    pub fn set_update_time(&self, update_time: u64) {
        self.0.write().unwrap().update_time = update_time;
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Eq, PartialEq, Clone)]
struct MemoryFileInner {
    pub create_time: u64,
    pub update_time: u64,
    pub buf: Vec<u8>,
}

impl MemoryFileInner {
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
