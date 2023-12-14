use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjectIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::object::ObjectMeta;
use crate::object::tree::Tree;

pub struct StageOp<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    trace: TraceIo<Fs, Io>,
    stage: StagingIo<Fs, Io>,
    object: ObjectIo<Fs, Io>,
    workspace: WorkspaceIo<Fs, Io>,
    commit: HeadIo<Fs, Io>,
    branch_name: BranchName,
}


impl<Fs, Io> StageOp<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn stage(&self, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.stage.read_tree()?.unwrap_or_default();
        // let now_tree = self.read_now_tree()?;
        // for obj in self.workspace.convert_to_objs(workspace_path)? {
        //     self.stage_file(&mut stage_tree, &now_tree, obj?)?;
        // }
        // self.stage.write_tree(&stage_tree)?;
        todo!();
        Ok(())
    }

    fn stage_file(&self, stage: &mut Tree, now: &Option<Tree>, meta: ObjectMeta) -> error::Result {
        if stage.changed_hash(&meta.file_path, meta.hash())
            || now
            .as_ref()
            .is_some_and(|now| now.changed_hash(&meta.file_path, meta.hash()))
        {
            self.object.write(&meta.obj)?;
            stage.insert(meta.file_path, meta.obj.hash);
        }
        Ok(())
    }
}