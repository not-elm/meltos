use std::io;

use crate::branch::BranchName;
use crate::io::{OpenIo, TvnIo};

#[derive(Debug, Clone)]
pub struct WorkBranchIo<Open, Io>(pub(crate) TvnIo<Open, Io>)
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write;

impl<Open, Io> Default for WorkBranchIo<Open, Io>
where
    Open: OpenIo<Io> + Default,
    Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvnIo::default())
    }
}

impl<Open, Io> WorkBranchIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    pub fn write(&self, branch_name: &BranchName) -> std::io::Result<()> {
        self.0
            .write(".meltos/WORK_BRANCH", &serde_json::to_vec(branch_name)?)?;
        Ok(())
    }

    pub fn read(&self) -> std::io::Result<BranchName> {
        let buf = self.0.try_read_to_end(".meltos/WORK_BRANCH")?;
        Ok(serde_json::from_slice(&buf)?)
    }
}
