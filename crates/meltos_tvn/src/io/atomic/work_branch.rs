use std::io;
use crate::branch::BranchName;
use crate::file_system::{FileSystem, FsIo};

#[derive(Debug, Clone)]
pub struct WorkBranchIo<Fs, Io>(pub(crate) FsIo<Fs, Io>)
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write;

impl<Fs, Io> Default for WorkBranchIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(FsIo::default())
    }
}

impl<Fs, Io> WorkBranchIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    pub fn write(&self, branch_name: &BranchName) -> std::io::Result<()> {
        self.0
            .write_all(".meltos/WORK_BRANCH", &serde_json::to_vec(branch_name)?)?;
        Ok(())
    }

    pub fn read(&self) -> std::io::Result<BranchName> {
        let buf = self.0.try_read_to_end(".meltos/WORK_BRANCH")?;
        Ok(serde_json::from_slice(&buf)?)
    }
}