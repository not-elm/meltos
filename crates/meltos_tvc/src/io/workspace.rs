use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FilePath, FileSystem};
use crate::io::atomic::head::HeadIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;
use crate::object::{AsMeta, Obj, ObjHash};

pub struct ChangeFileMeta {
    pub path: FilePath,
    pub change: ChangeFile,
}

pub enum ChangeFile {
    Create(FileObj),
    Update(FileObj),
    Delete,
}

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Fs>
where
    Fs: FileSystem,
{
    fs: Fs,
    head: HeadIo<Fs>,
    trace: TraceTreeIo<Fs>,
}

impl<Fs> WorkspaceIo<Fs>
where
    Fs: FileSystem + Clone,
{
    #[inline(always)]
    pub fn new(fs: Fs) -> WorkspaceIo<Fs> {
        Self {
            head: HeadIo::new(fs.clone()),
            trace: TraceTreeIo::new(fs.clone()),
            fs,
        }
    }
}

impl<Fs> WorkspaceIo<Fs>
where
    Fs: FileSystem,
{
    pub async fn try_read(&self, file_path: &FilePath) -> error::Result<FileObj> {
        match self.read(file_path).await? {
            Some(file_obj) => Ok(file_obj),
            None => Err(crate::error::Error::NotfoundWorkspaceFile(
                file_path.clone(),
            )),
        }
    }

    pub async fn read(&self, file_path: &FilePath) -> error::Result<Option<FileObj>> {
        let Some(buf) = self.fs.read_file(&self.as_path(file_path)).await? else {
            return Ok(None);
        };

        Ok(Some(FileObj(buf)))
    }

    pub async fn unpack(&self, file_path: &FilePath, obj: &Obj) -> error::Result<()> {
        match obj {
            Obj::File(file) => {
                self.fs.write_file(file_path, &file.0).await?;
                Ok(())
            }
            Obj::Delete(_) => {
                self.fs.delete(file_path).await?;
                Ok(())
            }
            _ => Err(crate::error::Error::InvalidWorkspaceObj),
        }
    }

    pub async fn is_change(&self, branch: &BranchName, path: &FilePath) -> error::Result<bool> {
        let head = self.head.try_read(branch).await?;
        let trace = self.trace.read(&head).await?;
        let file_obj = self.read(path).await?;

        if let Some(current_obj_hash) = trace.get(&FilePath(self.as_path(path))) {
            if let Some(file_obj) = file_obj {
                Ok(&file_obj.as_meta()?.hash != current_obj_hash)
            } else {
                Ok(true)
            }
        } else {
            Ok(file_obj.is_some())
        }
    }

    pub async fn convert_to_objs(&self, path: &str) -> error::Result<ObjectIter<Fs>> {
        let files = self.files(path).await?;

        Ok(ObjectIter {
            files,
            index: 0,
            io: &self.fs,
        })
    }

    #[inline(always)]
    pub async fn files(&self, path: &str) -> error::Result<Vec<String>> {
        let path = match path {
            "." | "./" => "/workspace".to_string(),
            path => format!("/workspace/{}", path.trim_start_matches("/workspace/")),
        };
        Ok(self.fs.all_files_in(&path).await?)
    }

    pub async fn changed_files(
        &self,
        mut trace_tree: TreeObj,
    ) -> error::Result<Vec<ChangeFileMeta>> {
        let mut changed_files = Vec::new();
        self.compare_trace(&mut trace_tree, &mut changed_files)
            .await?;
        for (path, _) in trace_tree.0.into_iter() {
            changed_files.push(ChangeFileMeta {
                path,
                change: ChangeFile::Delete,
            })
        }
        Ok(changed_files)
    }

    async fn compare_trace(
        &self,
        trace_tree: &mut TreeObj,
        changed_files: &mut Vec<ChangeFileMeta>,
    ) -> error::Result {
        let files = self.files(".").await?;
        for file_path in files {
            let path = FilePath(file_path);
            let file_obj = self.try_read(&path).await?;
            if let Some(trace_obj_hash) = trace_tree.remove(&path) {
                self.diff(changed_files, path, file_obj, trace_obj_hash)?;
            } else {
                changed_files.push(ChangeFileMeta {
                    path,
                    change: ChangeFile::Create(file_obj),
                });
            }
        }
        Ok(())
    }

    fn diff(
        &self,
        changed_files: &mut Vec<ChangeFileMeta>,
        path: FilePath,
        file_obj: FileObj,
        trace_obj_hash: ObjHash,
    ) -> error::Result {
        let meta = file_obj.as_meta()?;
        if meta.hash == trace_obj_hash {
            Ok(())
        } else {
            changed_files.push(ChangeFileMeta {
                path,
                change: ChangeFile::Update(file_obj),
            });
            Ok(())
        }
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        format!("/workspace/{}", path.trim_start_matches("/workspace/"))
    }
}

pub struct ObjectIter<'a, Fs>
where
    Fs: FileSystem,
{
    files: Vec<String>,
    index: usize,
    io: &'a Fs,
}

impl<'a, Fs> ObjectIter<'a, Fs>
where
    Fs: FileSystem,
{
    pub async fn next(&mut self) -> Option<std::io::Result<(FilePath, FileObj)>> {
        if self.index == self.files.len() {
            None
        } else {
            let obj = self.read_to_obj().await;
            self.index += 1;
            Some(obj)
        }
    }

    pub async fn all(&mut self) -> std::io::Result<Vec<(FilePath, FileObj)>> {
        let mut objs = Vec::new();
        while let Some(result) = self.next().await {
            objs.push(result?);
        }
        Ok(objs)
    }
}

impl<'a, Fs> ObjectIter<'a, Fs>
where
    Fs: FileSystem,
{
    async fn read_to_obj(&self) -> std::io::Result<(FilePath, FileObj)> {
        let path = self.files.get(self.index).unwrap();
        let buf = self.io.try_read_file(path.as_ref()).await?;
        Ok((FilePath::from_path(path), FileObj(buf)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::atomic::object::ObjIo;
    use crate::io::workspace::WorkspaceIo;
    use crate::object::file::FileObj;
    use crate::object::{AsMeta, Obj, ObjHash};
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn read_all_objects_in_dir() {
        let fs = MemoryFileSystem::default();
        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_file("/workspace/hello/hello.txt", b"hello")
            .await
            .unwrap();
        fs.write_file("/workspace/hello/world", b"world")
            .await
            .unwrap();
        fs.write_file("/workspace/hello/dir/main.sh", b"echo hi ")
            .await
            .unwrap();
        let mut hashes = workspace
            .convert_to_objs("hello")
            .await
            .unwrap()
            .all()
            .await
            .unwrap()
            .into_iter()
            .map(|obj| obj.1.clone().as_meta().unwrap().hash)
            .collect::<Vec<ObjHash>>();
        hashes.sort();
        let mut expect = vec![
            ObjHash::new(b"FILE\0hello"),
            ObjHash::new(b"FILE\0world"),
            ObjHash::new(b"FILE\0echo hi "),
        ];
        expect.sort();
        assert_eq!(hashes, expect);
    }

    #[tokio::test]
    async fn decode_buffer() {
        let fs = MemoryFileSystem::default();
        let workspace = WorkspaceIo::new(fs.clone());
        let obj = FileObj(b"hello".to_vec());
        let meta = obj.as_meta().unwrap();
        ObjIo::new(fs.clone())
            .write(&meta.hash, &meta.compressed_buf)
            .await
            .unwrap();
        workspace
            .unpack(&FilePath::from_path("hello.txt"), &Obj::File(obj))
            .await
            .unwrap();
        assert_eq!(fs.try_read_file("hello.txt").await.unwrap(), b"hello");
    }

    #[tokio::test]
    async fn read_all_files() {
        let fs = MemoryFileSystem::default();

        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_sync("/workspace/hello.txt", b"hello");
        fs.write_sync("/workspace/dist/index.js", b"index");
        let files = workspace.files(".").await.unwrap();
        assert_eq!(
            files.into_iter().collect::<HashSet<String>>(),
            vec![
                "/workspace/hello.txt".to_string(),
                "/workspace/dist/index.js".to_string(),
            ]
            .into_iter()
            .collect::<HashSet<String>>()
        );
    }

    #[tokio::test]
    async fn return_true_if_file_created() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_sync("/workspace/hello.txt", b"hello");

        let is_change = workspace
            .is_change(&BranchName::owner(), &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(is_change);
    }

    #[tokio::test]
    async fn return_true_if_file_changed() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_sync("/workspace/hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&branch, "").await.unwrap();
        fs.write_sync("/workspace/hello.txt", b"hello2");
        let is_change = workspace
            .is_change(&branch, &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(is_change);
    }

    #[tokio::test]
    async fn return_false_if_file_not_changed() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_sync("/workspace/hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&branch, "").await.unwrap();

        let is_change = workspace
            .is_change(&branch, &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(!is_change);
    }

    #[tokio::test]
    async fn return_true_if_file_deleted() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let workspace = WorkspaceIo::new(fs.clone());
        fs.write_sync("/workspace/hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&branch, "").await.unwrap();
        fs.delete("/workspace/hello.txt").await.unwrap();

        let is_change = workspace
            .is_change(&branch, &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(is_change);
    }

    #[tokio::test]
    async fn return_false_if_not_exists_both_workspace_and_traces() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let workspace = WorkspaceIo::new(fs.clone());

        let is_change = workspace
            .is_change(&branch, &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(!is_change);
    }

    #[tokio::test]
    async fn return_false_if_not_exists_both_workspace_and_traces2() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        let workspace = WorkspaceIo::new(fs.clone());

        fs.write_sync("/workspace/hello.txt", b"hello");
        fs.delete("/workspace/hello.txt").await.unwrap();

        let is_change = workspace
            .is_change(&branch, &FilePath("hello.txt".to_string()))
            .await
            .unwrap();
        assert!(!is_change);
    }
}
