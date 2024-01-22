use meltos::room::RoomId;
use std::path::{Path, PathBuf};

pub fn create_resource_dir(room_id: &RoomId) -> std::io::Result<()> {
    let path = room_resource_dir(room_id);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn delete_resource_dir(room_id: &RoomId) -> std::io::Result<()> {
    let path = room_resource_dir(room_id);
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

#[inline(always)]
pub fn room_resource_dir(room_id: &RoomId) -> PathBuf {
    Path::new("resources").join(&room_id.0)
}
