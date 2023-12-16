use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::ObjMeta;
use crate::object::tree::TreeObj;


#[derive(Debug, Clone)]
pub struct Stage<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    trace_tree: TraceTreeIo<Fs, Io>,
    staging: StagingIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    workspace: WorkspaceIo<Fs, Io>,
}


impl<Fs, Io> Stage<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    #[inline]
    pub fn new(branch_name: BranchName, fs: Fs) -> Stage<Fs, Io> {
        Self {
            staging: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name, fs.clone()),
            object: ObjIo::new(fs),
        }
    }
}


impl<Fs, Io> Stage<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn execute(&self, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.staging.read()?.unwrap_or_default();
        let trace_tree = self.trace_tree.read()?;
        for obj in self.workspace.convert_to_objs(workspace_path)? {
            self.stage_file(&mut stage_tree, &trace_tree, obj?)?;
        }
        self.staging.write_tree(&stage_tree)?;
        Ok(())
    }

    fn stage_file(&self, stage: &mut TreeObj, now: &Option<TreeObj>, meta: ObjMeta) -> error::Result {
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
    use crate::io::atomic::object::ObjIo;
    use crate::object::ObjHash;
    use crate::operation::stage::Stage;

    #[test]
    fn create_obj_file_after_staged() {
        let mock = MockFileSystem::default();
        let stage = Stage::new(BranchName::main(), mock.clone());
        mock.write(&FilePath::from_path("./hello"), b"hello").unwrap();
        mock.write(&FilePath::from_path("./src/main.rs"), "dasds日本語".as_bytes()).unwrap();
        stage.execute(".").unwrap();

        let obj = ObjIo::new(mock);
        let obj1 = obj.read_obj(&ObjHash::new(b"hello")).unwrap()
            .unwrap();
        assert_eq!(obj1.buf, b"hello");

        let obj2 = obj.read_obj(&ObjHash::new("dasds日本語".as_bytes())).unwrap().unwrap();
        assert_eq!(obj2.buf, "dasds日本語".as_bytes());
    }
}