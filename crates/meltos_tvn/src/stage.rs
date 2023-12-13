use crate::io::{FilePath, OpenIo, TvcIo};
use crate::tree::TreeIo;
use std::ops::Deref;


#[derive(Debug, Clone)]
pub struct StageIo<Open, Io>(pub(crate) TreeIo<Open, Io>)
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read;


impl<Open, Io> StageIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(open: Open) -> StageIo<Open, Io> {
        Self(TreeIo::new(
            FilePath::from("./.meltos/stage"),
            TvcIo::new(open),
        ))
    }
}


impl<Open, Io> Deref for StageIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    type Target = TreeIo<Open, Io>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<Open, Io> Default for StageIo<Open, Io>
where
    Open: OpenIo<Io> + Default,
    Io: std::io::Write + std::io::Read,
{
    fn default() -> Self {
        Self::new(Open::default())
    }
}
