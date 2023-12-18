use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::bundle::Bundle;
use crate::remote::CommitFetchable;

#[derive(Debug, Clone)]
pub struct Fetch<Fs, Io, Client>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    obj: ObjIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
    trace: TraceIo<Fs, Io>,
    client: Client,
}


impl<Fs, Io, Client> Fetch<Fs, Io, Client>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
        Client: CommitFetchable
{
    pub fn new(fs: Fs, client: Client) -> Fetch<Fs, Io, Client> {
        Self {
            obj: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs.clone()),
            trace: TraceIo::new(fs),
            client,
        }
    }
}


impl<Fs, Io, Client> Fetch<Fs, Io, Client>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
        Client: CommitFetchable
{
    pub async fn execute(&mut self, target_branch: Option<BranchName>) -> error::Result {
        let bundle = self.client.fetch(target_branch).await?;
        self.patch(&bundle)
    }

    fn patch(&self, bundle: &Bundle) -> error::Result {
        self.trace.write_all(&bundle.traces)?;
        for branch in &bundle.branches {
            self.head.write_remote(&branch.branch_name, &branch.head)?;
        }
        self.obj.write_all(&bundle.objs)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::head::HeadIo;
    use crate::io::atomic::object::ObjIo;
    use crate::io::atomic::trace::TraceIo;
    use crate::object::commit::CommitHash;
    use crate::operation::commit::Commit;
    use crate::operation::fetch::Fetch;
    use crate::operation::stage::Stage;
    use crate::remote::mock::MockRemoteClient;
    use crate::tests::init_main_branch;

    #[tokio::test]
    async fn updated_traces() {
        let mock = MockFileSystem::default();
        let (server, _) = create_mock_server_file_system();
        let mut fetch = Fetch::new(mock.clone(), server.clone());
        fetch.execute(None).await.unwrap();

        let mut server_traces = TraceIo::new(server.fs.clone()).read_all().unwrap();
        let mut local_traces = TraceIo::new(mock.clone()).read_all().unwrap();
        server_traces.sort();
        local_traces.sort();
        assert_eq!(server_traces, local_traces);
    }


    #[tokio::test]
    async fn updated_objs() {
        let mock = MockFileSystem::default();
        let (server, _) = create_mock_server_file_system();
        let mut fetch = Fetch::new(mock.clone(), server.clone());
        fetch.execute(None).await.unwrap();

        let mut server_objs = ObjIo::new(server.fs.clone()).read_all().unwrap();
        let mut local_objs = ObjIo::new(mock.clone()).read_all().unwrap();
        server_objs.sort();
        local_objs.sort();
        assert_eq!(server_objs, local_objs);
    }

    #[tokio::test]
    async fn updated_branches() {
        let mock = MockFileSystem::default();
        let (server, commit_hash) = create_mock_server_file_system();
        let mut fetch = Fetch::new(mock.clone(), server.clone());
        fetch.execute(None).await.unwrap();

        let remote_main_head = HeadIo::new(mock.clone())
            .read_remote(&BranchName::main())
            .unwrap();
        assert_eq!(remote_main_head, commit_hash);
    }

    fn create_mock_server_file_system() -> (MockRemoteClient, CommitHash) {
        let mock = MockFileSystem::default();
        mock.write("./hello.txt", b"hello").unwrap();
        init_main_branch(mock.clone());
        Stage::new(BranchName::main(), mock.clone()).execute(".").unwrap();
        let commit_hash = Commit::new(BranchName::main(), mock.clone()).execute("commit text").unwrap();
        (MockRemoteClient::new(mock), commit_hash)
    }
}
