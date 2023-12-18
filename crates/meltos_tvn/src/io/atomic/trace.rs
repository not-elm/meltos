use std::path::Path;

use crate::encode::{Decodable, Encodable};
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::object::commit::CommitHash;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct TraceIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
}

impl<Fs, Io> TraceIo<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> TraceIo<Fs, Io> {
        Self {
            io: FsIo::new(fs),
        }
    }


    pub fn write_all(&self, traces: &[(CommitHash, ObjHash)]) -> error::Result {
        for (commit, obj) in traces {
            self.write(commit, obj)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write(&self, commit_hash: &CommitHash, hash: &ObjHash) -> error::Result {
        let file_path = format!("./.meltos/traces/{commit_hash}");
        self.io.write(&file_path, &hash.encode()?)?;
        Ok(())
    }


    #[inline]
    pub fn read_all(&self) -> error::Result<Vec<(CommitHash, ObjHash)>> {
        let files = self.io.all_file_path("./.meltos/traces/")?;
        let mut traces = Vec::with_capacity(files.len());
        for file_path in files {
            let file_name = Path::new(&file_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let commit_hash = CommitHash(ObjHash(file_name));
            let buf = self
                .io
                .read(&file_path)?
                .ok_or(error::Error::NotfoundTrace(commit_hash.clone()))?;
            traces.push((commit_hash, ObjHash::decode(&buf)?));
        }

        Ok(traces)
    }


    #[inline]
    pub fn read(&self, commit_hash: &CommitHash) -> error::Result<ObjHash> {
        let file_path = format!("./.meltos/traces/{commit_hash}");
        let buf = self
            .io
            .try_read(&file_path)
            .map_err(|_| error::Error::NotfoundTrace(commit_hash.clone()))?;
        ObjHash::decode(&buf)
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::trace::TraceIo;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn read_all_traces() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let trace = TraceIo::new(mock.clone());
        let commit = Commit::new(branch, mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash1 = commit.execute("text").unwrap();

        mock.delete("./hello.txt").unwrap();
        stage.execute(".").unwrap();
        let commit_hash2 = commit.execute("text").unwrap();

        let traces = trace.read_all().unwrap();
        assert_eq!(traces.len(), 3);

        assert!(traces.iter().any(|(h, _)| h == &commit_hash1));
        assert!(traces.iter().any(|(h, _)| h == &commit_hash2));
    }
}
