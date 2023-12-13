use std::io::ErrorKind;

use crate::io::OpenIo;
use crate::now::NowIo;
use crate::object::{Object, ObjectIo};
use crate::stage::StageIo;
use crate::tree::Tree;
use crate::workspace::WorkspaceIo;

#[derive(Debug, Clone)]
pub struct BranchIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    stage: StageIo<Open, Io>,
    object: ObjectIo<Open, Io>,
    workspace: WorkspaceIo<Open, Io>,
    now: NowIo<Open, Io>
}


impl<Open, Io> BranchIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn stage(&self, workspace_path: &str) -> std::io::Result<()> {
        let mut stage = self.stage.read_tree()?.unwrap_or_default();
        for obj in self.workspace.convert_to_objs(workspace_path)? {
            self.stage_file(&mut stage, obj?)?;
        }
        self.stage.write_tree(&stage)?;
        Ok(())
    }

    pub fn commit(&self) -> std::io::Result<()> {
        let Some(_stage_tree) = self.stage.read_tree()? else {
            return Err(std::io::Error::new(ErrorKind::NotFound, "no staged files"));
        };
        self.stage.reset()?;
        Ok(())
    }

    fn stage_file(&self, stage: &mut Tree, object: Object) -> std::io::Result<()> {
        if stage.changed_hash(&object.file_path, &object.hash) {
            self.object.write(&object)?;
            stage.insert(object.file_path, object.hash);
        }
        Ok(())
    }
}


impl<Open, Io> Default for BranchIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: std::io::Write + std::io::Read
{
    fn default() -> Self {
        Self {
            stage: StageIo::default(),
            object: ObjectIo::default(),
            workspace: WorkspaceIo::default(),
            now: NowIo::default()
        }
    }
}