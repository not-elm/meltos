use std::ops::Deref;
use crate::file_system::{FilePath, FileSystem, FsIo};

use crate::object::tree::TreeIo;

#[derive(Debug, Clone)]
pub struct StagingIo<Fs, Io>(pub(crate) TreeIo<Fs, Io>)
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read;

impl<Fs, Io> StagingIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> StagingIo<Fs, Io> {
        Self(TreeIo::new(
            FilePath::from("./.meltos/stage"),
            FsIo::new(fs),
        ))
    }
}

impl<Fs, Io> Deref for StagingIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    type Target = TreeIo<Fs, Io>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Fs, Io> Default for StagingIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Default,
        Io: std::io::Write + std::io::Read,
{
    fn default() -> Self {
        Self::new(Fs::default())
    }
}
