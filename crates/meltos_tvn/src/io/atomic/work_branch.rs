use std::io;

use crate::branch::BranchName;
use crate::file_system::{FileSystem, FsIo};

#[derive(Debug, Clone)]
pub struct WorkingIo<Fs, Io>(pub(crate) FsIo<Fs, Io>)
where
    Fs: FileSystem<Io>,
    Io: io::Read + io::Write;

impl<Fs, Io> Default for WorkingIo<Fs, Io>
where
    Fs: FileSystem<Io> + Default,
    Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(FsIo::default())
    }
}

impl<Fs, Io> WorkingIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: io::Read + io::Write,
{
    #[inline]
    pub const fn new(fs: Fs) -> WorkingIo<Fs, Io> {
        Self(FsIo::new(fs))
    }


    #[inline]
    pub fn write(&self, branch_name: &BranchName) -> std::io::Result<()> {
        self.0
            .write(".meltos/WORKING", &serde_json::to_vec(branch_name)?)?;
        Ok(())
    }


    #[inline]
    pub fn try_read(&self) -> std::io::Result<BranchName> {
        let buf = self.0.try_read(".meltos/WORKING")?;
        Ok(serde_json::from_slice(&buf)?)
    }

    #[inline]
    pub fn read(&self) -> std::io::Result<Option<BranchName>> {
        let Some(buf) = self.0.read(".meltos/WORKING")? else {
            return Ok(None);
        };

        Ok(Some(serde_json::from_slice(&buf)?))
    }
}
