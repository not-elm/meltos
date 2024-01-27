use std::path::Path;

use crate::encode::{Decodable, Encodable};
use crate::error;
use crate::file_system::FileSystem;
use crate::io::bundle::BundleTrace;
use crate::object::commit::CommitHash;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct TraceIo<Fs>
where
    Fs: FileSystem,
{
    fs: Fs,
}

impl<Fs> TraceIo<Fs>
where
    Fs: FileSystem,
{
    #[inline(always)]
    pub const fn new(fs: Fs) -> TraceIo<Fs> {
        Self { fs }
    }

    pub async fn write_all(&self, traces: &[BundleTrace]) -> error::Result {
        for trace in traces {
            self.write(&trace.commit_hash, &trace.obj_hash).await?;
        }
        Ok(())
    }

    #[inline]
    pub async fn write(&self, commit_hash: &CommitHash, hash: &ObjHash) -> error::Result {
        let file_path = format!("/.meltos/traces/{commit_hash}");
        self.fs.write_file(&file_path, &hash.encode()?).await?;
        Ok(())
    }

    #[inline]
    pub async fn read_all(&self) -> error::Result<Vec<BundleTrace>> {
        let files = self.fs.all_files_in("/.meltos/traces/").await?;
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
                .fs
                .read_file(&file_path)
                .await?
                .ok_or(error::Error::NotfoundTrace(commit_hash.clone()))?;
            traces.push(BundleTrace {
                commit_hash,
                obj_hash: ObjHash::decode(&buf)?,
            });
        }

        Ok(traces)
    }

    #[inline]
    pub async fn read(&self, commit_hash: &CommitHash) -> error::Result<ObjHash> {
        let file_path = format!("/.meltos/traces/{commit_hash}");
        let buf = self
            .fs
            .try_read_file(&file_path)
            .await
            .map_err(|_| error::Error::NotfoundTrace(commit_hash.clone()))?;
        ObjHash::decode(&buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::trace::TraceIo;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn read_all_traces() -> error::Result {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;

        let branch = BranchName::owner();
        let stage = Stage::new(fs.clone());
        let trace = TraceIo::new(fs.clone());
        let commit = Commit::new(fs.clone());

        fs.write_file("/workspace/hello.txt", b"hello").await?;
        stage.execute(&branch, ".").await.unwrap();
        let commit_hash1 = commit.execute(&branch, "text").await.unwrap();

        fs.delete("/workspace/hello.txt").await?;
        stage.execute(&branch, ".").await.unwrap();
        let commit_hash2 = commit.execute(&branch, "text").await.unwrap();

        let traces = trace.read_all().await?;
        assert_eq!(traces.len(), 3);

        assert!(traces.iter().any(|trace| trace.commit_hash == commit_hash1));
        assert!(traces.iter().any(|trace| trace.commit_hash == commit_hash2));
        Ok(())
    }
}
