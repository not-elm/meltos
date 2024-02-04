use crate::branch::BranchName;
use crate::error;
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
    pub async fn write(&self, branch_name: &BranchName) -> error::Result<()> {
        self.0
            .write_file(".meltos/WORKING", &serde_json::to_vec(branch_name)?)
            .await?;
        Ok(())
    }

    #[inline]
    pub async fn try_read(&self) -> error::Result<BranchName> {
        let buf = self.0.try_read_file(".meltos/WORKING").await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    #[inline]
    pub async fn read(&self) -> error::Result<Option<BranchName>> {
        let Some(buf) = self.0.read_file(".meltos/WORKING").await? else {
            return Ok(None);
        };

        Ok(Some(serde_json::from_slice(&buf)?))
    }
}
