use crate::file_system::FileSystem;

mod conflict;
mod file;
mod merge;

#[derive(Debug, Clone)]
pub struct ObjDiff {
    old: String,
    new: String,
}

impl ObjDiff {}
