use std::time::{SystemTime, UNIX_EPOCH};

#[inline(always)]
pub fn since_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
