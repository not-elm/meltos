use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FilePath, FileSystem};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::io::workspace::WorkspaceIo;
use crate::object::{AsMeta, ObjHash};
use crate::object::delete::DeleteObj;
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;

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
}

impl<Fs> Stage<Fs>
    where
        Fs: FileSystem + Clone,
{
    #[inline]
    pub fn new(fs: Fs) -> Stage<Fs> {
        Self {
            staging: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
        }
    }
}

impl<Fs> Stage<Fs>
    where
        Fs: FileSystem,
{
    pub async fn execute(&self, branch_name: &BranchName, workspace_path: &str) -> error::Result {
        let mut stage_tree = self.staging.read().await?.unwrap_or_default();

        let trace_tree = {
            if let Some(head) = self.head.read(branch_name).await? {
                self.trace_tree.read(&head).await?
            } else {
                TreeObj::default()
            }
        };
        let mut changed = false;

        let mut objs = self.workspace.convert_to_objs(workspace_path).await?;
        while let Some(result) = objs.next().await {
            let (file_path, file_obj) = result?;
            self.stage_file(
                &mut stage_tree,
                &mut changed,
                &trace_tree,
                file_path,
                file_obj,
            ).await?;
        }

        self.add_delete_objs_into_staging(
            &mut stage_tree,
            &mut changed,
            &trace_tree,
            workspace_path,
        ).await?;

        if !changed {
            return Err(error::Error::ChangedFileNotExits);
        }

        self.staging.write_tree(&stage_tree).await?;
        Ok(())
    }

    async fn stage_file(
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
            self.object.write_obj(&file_obj).await?;
            stage.insert(file_path, meta.hash);
        }
        Ok(())
    }

    async fn add_delete_objs_into_staging(
        &self,
        staging: &mut TreeObj,
        changed: &mut bool,
        trace_tree: &TreeObj,
        work_space_path: &str,
    ) -> error::Result {
        for (path, hash) in self.scan_deleted_files(trace_tree, work_space_path).await? {
            *changed = true;
            let delete_obj = DeleteObj(hash);
            let delete_meta = delete_obj.as_meta()?;
            self.object.write_obj(&delete_obj).await?;
            staging.insert(path, delete_meta.hash);
        }
        Ok(())
    }

    async fn scan_deleted_files(
        &self,
        trace_tree: &TreeObj,
        workspace_path: &str,
    ) -> error::Result<Vec<(FilePath, ObjHash)>> {
        let work_space_files = self.workspace.files(".").await?;
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
    use crate::file_system::memory::MemoryFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::object::{AsMeta, ObjHash};
    use crate::object::delete::DeleteObj;
    use crate::object::file::FileObj;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn create_obj_file_after_staged() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let stage = Stage::new(fs.clone());
        fs
            .write_file(&FilePath::from_path("workspace/hello"), b"hello")
            .await
            .unwrap();
        fs
            .write_file(
                &FilePath::from_path("workspace/src/main.rs"),
                "dasds日本語".as_bytes(),
            )
            .await
            .unwrap();
        stage.execute(&branch, ".").await.unwrap();

        let obj = ObjIo::new(fs);
        let obj1 = obj
            .read_obj(&ObjHash::new(b"FILE\0hello"))
            .await
            .unwrap()
            .unwrap()
            .file()
            .unwrap()
            .0;

        assert_eq!(obj1, b"hello");

        let obj2 = obj
            .read_obj(&ObjHash::new("FILE\0dasds日本語".as_bytes()))
            .await
            .unwrap()
            .unwrap()
            .file()
            .unwrap()
            .0;
        assert_eq!(obj2, "dasds日本語".as_bytes());
    }

    #[tokio::test]
    async fn create_delete_obj() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());

        fs.write_file("workspace/hello.txt", b"hello").await.unwrap();
        stage.execute(&branch, "hello.txt").await.unwrap();
        commit.execute(&branch, "add hello.txt").await.unwrap();

        fs.delete("workspace/hello.txt").await.unwrap();
        stage.execute(&branch, "hello.txt").await.unwrap();
        commit.execute(&branch, "delete hello.txt").await.unwrap();

        let hello_hash = FileObj(b"hello".to_vec()).as_meta().unwrap().hash;
        let delete_hello = DeleteObj(hello_hash).as_meta().unwrap();
        let delete_hello_hash = delete_hello.hash;

        let buf = ObjIo::new(fs)
            .read_obj(&delete_hello_hash)
            .await
            .unwrap()
            .unwrap()
            .as_meta()
            .unwrap()
            .buf;
        assert_eq!(buf, delete_hello.buf);
    }

    #[tokio::test]
    async fn no_moved_if_not_changed_file() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;

        let branch = BranchName::owner();
        let stage = Stage::new(fs.clone());

        fs.write_file("workspace/hello.txt", b"hello").await.unwrap();
        stage.execute(&branch, ".").await.unwrap();

        match stage.execute(&branch, ".").await {
            Err(error::Error::ChangedFileNotExits) => {}
            _ => panic!("expected the [error::Error::ChangedFileNotExits] bad was."),
        }
    }
}
