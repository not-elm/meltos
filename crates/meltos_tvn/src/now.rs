use crate::branch::BranchName;
use crate::error;
use crate::io::{OpenIo, TvnIo};
use crate::object::ObjectHash;

#[derive(Debug, Clone)]
pub struct NowIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: TvnIo<Open, Io>,
    file_path: String,
}

impl<Open, Io> NowIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, open: Open) -> NowIo<Open, Io> {
        Self {
            file_path: format!("./.meltos/branches/{}/NOW", branch_name),
            io: TvnIo::new(open),
        }
    }

    #[inline]
    pub fn write_hash(&self, hash: &ObjectHash) -> error::Result {
        self.io.write(&self.file_path, &hash.serialize_to_buf())?;
        Ok(())
    }

    #[inline]
    pub fn read_hash(&self) -> error::Result<Option<ObjectHash>> {
        let Some(buf) = self.io.read_to_end(&self.file_path)? else {
            return Ok(None);
        };
        Ok(Some(ObjectHash::from_serialized_buf(&buf)?))
    }

    #[inline]
    pub fn exists(&self) -> error::Result<bool> {
        Ok(self.read_hash()?.is_some())
    }
}
