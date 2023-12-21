use crate::file_system::FileSystem;

mod file;
mod conflict;
mod merge;

#[derive(Debug, Clone)]
pub struct ObjDiff {
    old: String,
    new: String,
}

impl ObjDiff {}



