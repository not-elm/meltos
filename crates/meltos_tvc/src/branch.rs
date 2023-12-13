use std::io::ErrorKind;
use std::path::Path;

use crate::io::{FilePath, OpenIo};
use crate::object::ObjectIo;
use crate::stage::StageIo;
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
}


impl<Open, Io> BranchIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn stage(&self, workspace_path: &str) -> std::io::Result<()> {
        let path = Path::new(workspace_path);
        if path.is_dir() {
            for entry in path.read_dir()? {
                let entry_path = entry?.path();
                let Some(entry_path) = entry_path.to_str() else {
                    continue;
                };
                self.stage(entry_path)?;
            }
            Ok(())
        } else {
            self.stage_file(workspace_path)
        }
    }

    pub fn commit(&self) -> std::io::Result<()> {
        let Some(stage_tree) = self.stage.read_tree()? else {
            return Err(std::io::Error::new(ErrorKind::NotFound, "no staged files"));
        };
        self.stage.reset()?;

        Ok(())
    }

    fn stage_file(&self, workspace_path: &str) -> std::io::Result<()> {
        let object = self.workspace.read_to_object(workspace_path)?;
        if self.stage.check_changed_file(FilePath::from(workspace_path), object.hash.clone())? {
            self.object.write(&object)?;
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
        }
    }
}