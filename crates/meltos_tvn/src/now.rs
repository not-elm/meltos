use std::ops::Deref;

use crate::branch::BranchName;
use crate::io::{FilePath, OpenIo, TvnIo};
use crate::tree::TreeIo;

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
            FilePath::from(format!("./.meltos/branches/{branch_name}/NOW")),
            TvnIo::new(open),
        ))
    }


    #[inline]
    pub fn exists(&self) -> std::io::Result<bool> {
        Ok(self.read_tree()?.is_some())
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
