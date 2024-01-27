use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::io::workspace::WorkspaceIo;

#[derive(Debug, Clone)]
pub struct UnZip<Fs>
where
    Fs: FileSystem,
{
    workspace: WorkspaceIo<Fs>,
    trace_tree: TraceTreeIo<Fs>,
    object: ObjIo<Fs>,
    head: HeadIo<Fs>,
    fs: Fs,
}

impl<Fs> UnZip<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> UnZip<Fs> {
        Self {
            workspace: WorkspaceIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(fs.clone()),
            fs,
        }
    }
}

impl<Fs> UnZip<Fs>
where
    Fs: FileSystem,
{
    /// Restore committed data into the workspace.
    pub async fn execute(&self, branch_name: &BranchName) -> error::Result {
        self.fs.delete("/workspace").await?;
        let head = self.head.try_read(branch_name).await?;
        let trace_tree = self.trace_tree.read(&head).await?;
        for (path, hash) in trace_tree.iter() {
            self.workspace
                .unpack(path, &self.object.try_read_obj(hash).await?)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::operation::unzip::UnZip;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn success_if_committed() -> error::Result {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();

        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let unzip = UnZip::new(fs.clone());

        fs.write_file("/workspace/hello", b"hello").await?;
        stage.execute(&branch, "hello").await?;
        commit.execute(&branch, "commit text").await?;
        fs.delete("/workspace/hello").await?;

        unzip.execute(&branch).await?;
        assert_eq!(fs.try_read_file("/workspace/hello").await?, b"hello");
        Ok(())
    }
}
