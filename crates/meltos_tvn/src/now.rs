use crate::branch::BranchName;
use crate::io::{FilePath, OpenIo, TvnIo};
use crate::tree::TreeIo;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct NowIo<Open, Io>(TreeIo<Open, Io>)
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read;


impl<Open, Io> NowIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, open: Open) -> NowIo<Open, Io> {
        Self(TreeIo::new(
            FilePath::from(format!("./.meltos/{branch_name}/now")),
            TvnIo::new(open),
        ))
    }
}

impl<Open, Io> Deref for NowIo<Open, Io>
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
