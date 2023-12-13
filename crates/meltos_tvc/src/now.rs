use std::ops::Deref;
use crate::io::{FilePath, OpenIo, TvcIo};
use crate::tree::TreeIo;

#[derive(Debug, Clone)]
pub struct NowIo<Open, Io>(TreeIo<Open, Io>)
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read;


impl<Open, Io> Deref for NowIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    type Target = TreeIo<Open, Io>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



impl<Open, Io> Default for NowIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: std::io::Write + std::io::Read
{
    fn default() -> Self {
        Self(TreeIo::new(FilePath::from("./.meltos/now"), TvcIo::default()))
    }
}
