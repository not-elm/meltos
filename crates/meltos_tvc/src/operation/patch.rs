use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::bundle::Bundle;

#[derive(Debug, Clone)]
pub struct Patch<Fs>
    where
        Fs: FileSystem,
{
    obj: ObjIo<Fs>,
    head: HeadIo<Fs>,
    trace: TraceIo<Fs>,
}

impl<Fs> Patch<Fs>
    where
        Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Patch<Fs> {
        Self {
            obj: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace: TraceIo::new(fs),
        }
    }
}

impl<Fs> Patch<Fs>
    where
        Fs: FileSystem,
{
    pub async fn execute(&self, bundle: &Bundle) -> error::Result {
        self.trace.write_all(&bundle.traces).await?;
        for branch in &bundle.branches {
            self
                .head
                .write_remote(
                    &branch.branch_name,
                    &branch.commits[branch.commits.len() - 1],
                )
                .await?;
        }
        self.obj.write_all(&bundle.objs).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use crate::branch::BranchName;
    // use crate::file_system::FileSystem;
    // use crate::file_system::mock::MockFileSystem;
    // use crate::io::atomic::head::HeadIo;
    // use crate::io::atomic::object::ObjIo;
    // use crate::io::atomic::trace::TraceIo;
    // use crate::io::bundle::BundleIo;
    // use crate::object::commit::CommitHash;
    // use crate::operation::commit::Commit;
    // use crate::operation::patch::Patch;
    // use crate::operation::stage::Stage;
    //
    // use crate::tests::init_main_branch;
    // //
    // #[tokio::test]
    // async fn updated_traces() {
    //     let fs = MockFileSystem::default();
    //     let (server, _) = create_mock_server_file_system();
    //     let bundle = BundleIo::new(fs.clone()).create().unwrap();
    //     let mut fetch = Patch::new(fs.clone());
    //     fetch.execute(&bundle).unwrap();
    //
    //     let mut server_traces = TraceIo::new(server.fs.clone()).read_all().unwrap();
    //     let mut local_traces = TraceIo::new(fs.clone()).read_all().unwrap();
    //     server_traces.sort();
    //     local_traces.sort();
    //     assert_eq!(server_traces, local_traces);
    // }
    //
    //
    // #[tokio::test]
    // async fn updated_objs() {
    //     let fs = MockFileSystem::default();
    //     let (server, _) = create_mock_server_file_system();
    //     let bundle = BundleIo::new(fs.clone()).create().unwrap();
    //     let mut fetch = Patch::new(fs.clone());
    //     fetch.execute(&bundle).unwrap();
    //
    //     let mut server_objs = ObjIo::new(server.fs.clone()).read_all().unwrap();
    //     let mut local_objs = ObjIo::new(fs.clone()).read_all().unwrap();
    //     server_objs.sort();
    //     local_objs.sort();
    //     assert_eq!(server_objs, local_objs);
    // }
    //
    // #[tokio::test]
    // async fn updated_branches() {
    //     let fs = MockFileSystem::default();
    //     let (server, commit_hash) = create_mock_server_file_system();
    //     let bundle = BundleIo::new(fs.clone()).create().unwrap();
    //     let mut patch = Patch::new(fs.clone());
    //     patch.execute(&bundle).unwrap();
    //
    //     let remote_main_head = HeadIo::new(fs.clone())
    //         .try_read_remote(&BranchName::main())
    //         .unwrap();
    //     assert_eq!(remote_main_head, commit_hash);
    // }
    //
    // fn create_mock_server_file_system() -> (MockRemoteClient, CommitHash) {
    //     let fs = MockFileSystem::default();
    //     mock.write("./hello.txt", b"hello").unwrap();
    //     init_main_branch(fs.clone());
    //     Stage::new(BranchName::main(), fs.clone())
    //         .execute(".")
    //         .unwrap();
    //     let commit_hash = Commit::new(BranchName::main(), fs.clone())
    //         .execute("commit text")
    //         .unwrap();
    //     (MockRemoteClient::new(mock), commit_hash)
    // }
}
