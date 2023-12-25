use crate::branch::BranchName;
use crate::file_system::FileSystem;

#[derive(Debug, Clone, Default)]
pub struct WorkingIo<Fs>(pub(crate) Fs)
where
    Fs: FileSystem;

impl<Fs> WorkingIo<Fs>
where
    Fs: FileSystem,
{
    #[inline]
    pub const fn new(fs: Fs) -> WorkingIo<Fs> {
        Self(fs)
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
