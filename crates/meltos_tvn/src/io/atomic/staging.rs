use std::ops::Deref;

use crate::file_system::{FilePath, FileSystem, };
use crate::object::tree::TreeIo;

#[derive(Debug, Clone)]
pub struct StagingIo<Fs>(pub(crate) TreeIo<Fs>)
    where
        Fs: FileSystem;

impl<Fs> StagingIo<Fs>
    where
        Fs: FileSystem
{
    pub fn new(fs: Fs) -> StagingIo<Fs> {
        Self(TreeIo::new(
            FilePath::from("./.meltos/stage"),
            fs,
        ))
    }
}

impl<Fs> Deref for StagingIo<Fs>
    where
        Fs: FileSystem
{
    type Target = TreeIo<Fs>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

