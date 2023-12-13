use std::io::ErrorKind;
use meltos_util::compression::gz::Gz;
use crate::io::{FilePath, OpenIo};
use crate::io::compression::{CompressionIo, CompressionOpen};
use crate::object::ObjectIo;
use crate::stage::StageIo;

#[derive(Debug, Clone)]
pub struct BranchIo<Open, Io>
    where
        Open: OpenIo<Io> ,
        Io: std::io::Write + std::io::Read
{
    stage: StageIo<Open, Io>,
    object: ObjectIo<CompressionOpen<Open, Io, Gz>, CompressionIo<Io, Gz>>,
    io: Open
}


impl<Open, Io> BranchIo<Open, Io>
    where
        Open: OpenIo<Io> ,
        Io: std::io::Write + std::io::Read
{
    pub fn stage(&self, stage_file_path: &str) -> std::io::Result<()> {
        let buf = self.io.try_read_to_end(stage_file_path)?;
        let object_hash = self.object.write(buf)?;
        self.stage.write_object_hash(FilePath::from(stage_file_path), object_hash)?;
        Ok(())
    }

    pub fn commit(&self) -> std::io::Result<()> {
        let Some(stage_tree) = self.stage.read_tree()? else{
            return Err(std::io::Error::new(ErrorKind::NotFound, "no staged files"))
        };
        self.stage.reset()?;

        Ok(())
    }
}


impl<Open, Io> Default for BranchIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: std::io::Write + std::io::Read
{
    fn default() -> Self {
        Self{
            stage: StageIo::default(),
            object: ObjectIo::default(),
            io: Open::default()
        }
    }
}