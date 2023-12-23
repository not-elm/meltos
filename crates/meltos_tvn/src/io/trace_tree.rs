use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::commit::CommitHash;
use crate::object::tree::TreeObj;
use crate::object::AsMeta;

#[derive(Debug, Clone)]
pub struct TraceTreeIo<Fs>
where
    Fs: FileSystem
{
    trace: TraceIo<Fs>,
    object: ObjIo<Fs>,
}

impl<Fs> TraceTreeIo<Fs>
where
    Fs: FileSystem + Clone
{
    pub fn new(fs: Fs) -> TraceTreeIo<Fs> {
        Self {
            trace: TraceIo::new(fs.clone()),
            object: ObjIo::new(fs),
        }
    }
}

impl<Fs> TraceTreeIo<Fs>
where
    Fs: FileSystem
{
    pub fn write(&self, trace_tree: &TreeObj, commit_hash: &CommitHash) -> error::Result {
        let meta = trace_tree.as_meta()?;
        self.trace.write(commit_hash, &meta.hash)?;
        self.object.write_obj(trace_tree)?;
        Ok(())
    }

    pub fn read(&self, commit_hash: &CommitHash) -> error::Result<TreeObj> {
        let trace_hash = self.trace.read(commit_hash)?;
        let trace_tree = self.object.read_to_tree(&trace_hash)?;
        Ok(trace_tree)
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::encode::Encodable;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::trace_tree::TraceTreeIo;
    use crate::object::commit::CommitHash;
    use crate::object::tree::TreeObj;
    use crate::object::{AsMeta, ObjHash};

    use crate::operation::init;

    #[test]
    fn success_read_trace_tree() {
        let mock = MockFileSystem::default();
        let init = init::Init::new(BranchName::main(), mock.clone());
        let trace_tree = TraceTreeIo::new(mock.clone());
        init.execute().unwrap();

        let mut tree = TreeObj::default();
        tree.insert(FilePath::from("me/hello"), ObjHash::new(b"hello"));

        let commit_hash = CommitHash(ObjHash::new(b"commit"));
        trace_tree.write(&tree, &commit_hash).unwrap();
        mock.write(
            &format!("./.meltos/branches/traces/{commit_hash}"),
            &tree.as_meta().unwrap().hash.encode().unwrap(),
        )
        .unwrap();
        trace_tree.read(&commit_hash).unwrap();
    }

    #[test]
    fn read_tree_after_wrote() {
        let mock = MockFileSystem::default();
        let trace_tre = TraceTreeIo::new(mock);
        let mut staging = TreeObj::default();
        staging.insert(FilePath::from_path("./src/hello"), ObjHash::new(b"hello"));
        let commit_hash = CommitHash(ObjHash::new(b"commit"));
        trace_tre.write(&staging, &commit_hash).unwrap();
        let tree_obj = trace_tre.read(&commit_hash).unwrap();
        assert_eq!(tree_obj, staging);
    }
}
