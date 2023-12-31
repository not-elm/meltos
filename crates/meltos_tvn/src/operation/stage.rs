use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FilePath, FileSystem};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::io::workspace::WorkspaceIo;
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;
use crate::object::{AsMeta, ObjHash};

#[derive(Debug, Clone)]
pub struct Stage<Fs>
where
    Fs: FileSystem,
{
    trace_tree: TraceTreeIo<Fs>,
    staging: StagingIo<Fs>,
    object: ObjIo<Fs>,
    head: HeadIo<Fs>,
    workspace: WorkspaceIo<Fs>,
    branch_name: BranchName,
}

impl<Fs> Stage<Fs>
where
    Fs: FileSystem + Clone,
{
    #[inline]
    pub fn new(branch_name: BranchName, fs: Fs) -> Stage<Fs> {
        Self {
            staging: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            branch_name,
        }
    }
}

impl<Fs> Stage<Fs>
where
    Fs: FileSystem,
{
    pub fn execute(&self, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.staging.read()?.unwrap_or_default();
        let trace_tree = {
            if let Some(head) = self.head.read(&self.branch_name)? {
                self.trace_tree.read(&head)?
            } else {
                TreeObj::default()
            }
        };
        let mut changed = false;
        for result in self.workspace.convert_to_objs(workspace_path)? {
            let (file_path, file_obj) = result?;
            self.stage_file(
                &mut stage_tree,
                &mut changed,
                &trace_tree,
                file_path,
                file_obj,
            )?;
        }
        self.add_delete_objs_into_staging(
            &mut stage_tree,
            &mut changed,
            &trace_tree,
            workspace_path,
        )?;
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
        let meta = file_obj.as_meta()?;
        if !trace.changed_hash(&file_path, &meta.hash) {
            return Ok(());
        }

        if stage.changed_hash(&file_path, &meta.hash) {
            *changed = true;
            self.object.write_obj(&file_obj)?;
            stage.insert(file_path, meta.hash);
        }
        Ok(())
    }

    fn add_delete_objs_into_staging(
        &self,
        staging: &mut TreeObj,
        changed: &mut bool,
        trace_tree: &TreeObj,
        work_space_path: &str,
    ) -> error::Result {
        for (path, hash) in self.scan_deleted_files(trace_tree, work_space_path)? {
            *changed = true;
            let delete_obj = DeleteObj(hash);
            let delete_meta = delete_obj.as_meta()?;
            self.object.write_obj(&delete_obj)?;
            staging.insert(path, delete_meta.hash);
        }
        Ok(())
    }

    fn scan_deleted_files(
        &self,
        trace_tree: &TreeObj,
        workspace_path: &str,
    ) -> error::Result<Vec<(FilePath, ObjHash)>> {
        let work_space_files = self.workspace.files(workspace_path)?;
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
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::atomic::object::ObjIo;
    use crate::object::delete::DeleteObj;
    use crate::object::file::FileObj;
    use crate::object::{AsMeta, ObjHash};
    use crate::operation::commit::Commit;
    use crate::operation::stage;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn create_obj_file_after_staged() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let stage = Stage::new(BranchName::main(), mock.clone());
        mock.write(&FilePath::from_path("./workspace/hello"), b"hello")
            .unwrap();
        mock.write(
            &FilePath::from_path("./workspace/src/main.rs"),
            "dasds日本語".as_bytes(),
        )
        .unwrap();
        stage.execute(".").unwrap();

        let obj = ObjIo::new(mock);
        let obj1 = obj
            .read_obj(&ObjHash::new(b"FILE\0hello"))
            .unwrap()
            .unwrap()
            .file()
            .unwrap()
            .0;

        assert_eq!(obj1, b"hello");

        let obj2 = obj
            .read_obj(&ObjHash::new("FILE\0dasds日本語".as_bytes()))
            .unwrap()
            .unwrap()
            .file()
            .unwrap()
            .0;
        assert_eq!(obj2, "dasds日本語".as_bytes());
    }

    #[test]
    fn create_delete_obj() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let stage = Stage::new(BranchName::main(), mock.clone());
        let commit = Commit::new(BranchName::main(), mock.clone());

        mock.write("./workspace/hello.txt", b"hello").unwrap();
        stage.execute("hello.txt").unwrap();
        commit.execute("add hello.txt").unwrap();

        mock.delete("./workspace/hello.txt").unwrap();
        stage.execute("hello.txt").unwrap();
        commit.execute("delete hello.txt").unwrap();

        let hello_hash = FileObj(b"hello".to_vec()).as_meta().unwrap().hash;
        let delete_hello = DeleteObj(hello_hash).as_meta().unwrap();
        let delete_hello_hash = delete_hello.hash;

        let buf = ObjIo::new(mock)
            .read_obj(&delete_hello_hash)
            .unwrap()
            .unwrap()
            .as_meta()
            .unwrap()
            .buf;
        assert_eq!(buf, delete_hello.buf);
    }

    #[test]
    fn no_moved_if_not_changed_file() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let stage = stage::Stage::new(BranchName::main(), mock.clone());

        mock.write("./workspace/hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();

        match stage.execute(".") {
            Err(error::Error::ChangedFileNotExits) => {}
            _ => panic!("expected the [error::Error::ChangedFileNotExits] bad was."),
        }
    }
}
