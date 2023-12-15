use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjectIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::ObjectMeta;
use crate::object::tree::Tree;

pub struct StageIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    trace_tree: TraceTreeIo<Fs, Io>,
    staging: StagingIo<Fs, Io>,
    object: ObjectIo<Fs, Io>,
    workspace: WorkspaceIo<Fs, Io>,
}


impl<Fs, Io> StageIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    #[inline]
    pub fn new(branch_name: BranchName, fs: Fs) -> StageIo<Fs, Io> {
        Self {
            staging: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name, fs.clone()),
            object: ObjectIo::new(fs),
        }
    }
}


impl<Fs, Io> StageIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn stage(&self, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.staging.read_tree()?.unwrap_or_default();
        let trace_tree = self.trace_tree.read_trace_tree()?;
        for obj in self.workspace.convert_to_objs(workspace_path)? {
            self.stage_file(&mut stage_tree, &trace_tree, obj?)?;
        }
        self.staging.write_tree(&stage_tree)?;
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


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjectIo;
    use crate::io::stage::StageIo;
    use crate::object::ObjectHash;

    #[test]
    fn create_obj_file_after_staged() {
        let mock = MockFileSystem::default();
        let stage = StageIo::new(BranchName::main(), mock.clone());
        mock.write_all(&FilePath::from_path("./hello"), b"hello").unwrap();
        mock.write_all(&FilePath::from_path("./src/main.rs"), "dasds日本語".as_bytes()).unwrap();
        stage.stage(".").unwrap();

        let obj = ObjectIo::new(mock);
        let obj1 = obj.read_obj(&ObjectHash::new(b"hello")).unwrap()
            .unwrap();
        assert_eq!(obj1.buf, b"hello");

        let obj2 = obj.read_obj(&ObjectHash::new("dasds日本語".as_bytes())).unwrap().unwrap();
        assert_eq!(obj2.buf, "dasds日本語".as_bytes());
    }
}