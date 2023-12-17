use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FilePath, FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::{AsMeta, ObjHash};
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
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
    head: HeadIo<Fs, Io>,
    workspace: WorkspaceIo<Fs, Io>,
    fs: FsIo<Fs, Io>,
}


impl<Fs, Io> Stage<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new(branch_name: BranchName, fs: Fs) -> Stage<Fs, Io> {
        Self {
            staging: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            head: HeadIo::new(branch_name, fs.clone()),
            object: ObjIo::new(fs.clone()),
            fs: FsIo::new(fs),
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
        let head = self.head.read()?;
        let trace_tree = self.trace_tree.read(&head)?;
        let mut changed = false;
        for result in self.workspace.convert_to_objs(workspace_path)? {
            let (file_path, file_obj) = result?;
            self.stage_file(&mut stage_tree, &mut changed, &trace_tree, file_path, file_obj)?;
        }
        self.add_delete_objs_into_staging(&mut stage_tree, &mut changed, &trace_tree, workspace_path)?;
        if !changed {
            return Err(error::Error::ChangedFileNotExits);
        }

        self.staging.write_tree(&stage_tree)?;
        Ok(())
    }


    fn stage_file(
        &self,
        stage: &mut TreeObj,
        changed: &mut bool,
        trace: &TreeObj,
        file_path: FilePath,
        file_obj: FileObj,
    ) -> error::Result {
        let obj = file_obj.as_meta()?;
        if !trace.changed_hash(&file_path, &obj.hash) {
            return Ok(());
        }

        if stage.changed_hash(&file_path, &obj.hash) {
            *changed = true;
            self.object.write(&obj)?;
            stage.insert(file_path, obj.hash);
        }
        Ok(())
    }


    fn add_delete_objs_into_staging(
        &self,
        staging: &mut TreeObj,
        changed: &mut bool,
        trace_tree: &TreeObj,
        work_space_path: &str) -> error::Result {
        for (path, hash) in self.scan_deleted_files(trace_tree, work_space_path)? {
            *changed = true;
            let delete_obj = DeleteObj(hash).as_meta()?;
            self.object.write(&delete_obj)?;
            staging.insert(path, delete_obj.hash);
        }
        Ok(())
    }


    fn scan_deleted_files(
        &self,
        trace_tree: &TreeObj,
        workspace_path: &str,
    ) -> error::Result<Vec<(FilePath, ObjHash)>> {
        let work_space_files = self.fs.all_file_path(workspace_path)?;
        Ok(trace_tree
            .iter()
            .filter_map(|(path, hash)| {
                if work_space_files.contains(&path.0) {
                    None
                } else {
                    Some((path.clone(), hash.clone()))
                }
            })
            .collect())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::object::{AsMeta, ObjHash};
    use crate::object::delete::DeleteObj;
    use crate::object::file::FileObj;
    use crate::operation::commit::Commit;
    use crate::operation::stage;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn create_obj_file_after_staged() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        mock.write(&FilePath::from_path("./hello"), b"hello")
            .unwrap();
        mock.write(
            &FilePath::from_path("./src/main.rs"),
            "dasds日本語".as_bytes(),
        )
            .unwrap();
        stage.execute(".").unwrap();

        let obj = ObjIo::new(mock);
        let obj1 = obj.read_obj(&ObjHash::new(b"hello")).unwrap().unwrap();
        assert_eq!(obj1.buf, b"hello");

        let obj2 = obj
            .read_obj(&ObjHash::new("dasds日本語".as_bytes()))
            .unwrap()
            .unwrap();
        assert_eq!(obj2.buf, "dasds日本語".as_bytes());
    }


    #[test]
    fn create_delete_obj() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute("./hello.txt").unwrap();
        commit.execute("add hello.txt").unwrap();

        mock.delete("./hello.txt").unwrap();
        stage.execute("./hello.txt").unwrap();
        commit.execute("delete hello.txt").unwrap();

        let hello_hash = FileObj(b"hello".to_vec()).as_meta().unwrap().hash;
        let delete_hello = DeleteObj(hello_hash).as_meta().unwrap();
        let delete_hello_hash = delete_hello.hash;

        let buf = ObjIo::new(mock)
            .read_obj(&delete_hello_hash)
            .unwrap()
            .unwrap()
            .buf;
        assert_eq!(buf, delete_hello.buf);
    }


    #[test]
    fn no_moved_if_not_changed_file() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let stage = stage::Stage::new(BranchName::main(), mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();

        match stage.execute(".") {
            Err(error::Error::ChangedFileNotExits) => {}
            _ => panic!("expected the [error::Error::ChangedFileNotExits] bad was.")
        }
    }
}
