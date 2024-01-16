use std::path::{Path, PathBuf};
use meltos::room::RoomId;



pub fn create_resource_dir(room_id: &RoomId) -> std::io::Result<()>{
    let path = room_resource_dir(room_id);
    if !path.exists(){
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}


pub fn delete_resource_dir(room_id: &RoomId) -> std::io::Result<()>{
    let path = room_resource_dir(room_id);
    if path.exists(){
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}


pub fn room_resource_dir(room_id: &RoomId) -> PathBuf {
    Path::new("resources").join(&room_id.0)
}