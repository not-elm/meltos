use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::tree::TreeObj;


#[derive(Debug, Clone)]
pub struct TraceTreeIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    trace: TraceIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
}


impl<Fs, Io> TraceTreeIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> TraceTreeIo<Fs, Io> {
        Self {
            trace: TraceIo::new(branch_name, fs.clone()),
            object: ObjIo::new(fs),
        }
    }
}

impl<Fs, Io> TraceTreeIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn write(&self, staging: TreeObj) -> error::Result {
        let mut trace_tree = self
            .read()?
            .unwrap_or_default();
        trace_tree.replace_by(staging);
        let trace_obj = trace_tree.as_obj()?;
        self.trace.write(&trace_obj.hash)?;
        self.object.write(&trace_obj)?;
        Ok(())
    }


    pub fn read(&self) -> error::Result<Option<TreeObj>> {
        let Some(trace_hash) = self.trace.read_hash()?
            else {
                return Ok(None);
            };
        let tree = self.object.read_to_tree(&trace_hash)?;
        Ok(Some(tree))
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::trace_tree::TraceTreeIo;
    use crate::object::ObjHash;
    use crate::object::tree::TreeObj;

    #[test]
    fn success_read_trace_tree() {
        let mock = MockFileSystem::default();
        let io = TraceTreeIo::new(BranchName::main(), mock.clone());
        let mut tree = TreeObj::default();
        tree.insert(FilePath::from("me/hello"), ObjHash::new(b"hello"));
        let obj = ObjIo::new(mock.clone());
        obj.write(&tree.as_obj().unwrap()).unwrap();

        mock.write("./.meltos/branches/main/TRACE", &tree.as_obj().unwrap().hash.serialize_to_buf()).unwrap();
        let trace_tree = io.read();
        assert!(trace_tree.is_ok_and(|tree| tree.is_some()));
    }


    #[test]
    fn read_tree_after_wrote() {
        let mock = MockFileSystem::default();
        let io = TraceTreeIo::new(BranchName::main(), mock);
        let mut staging = TreeObj::default();
        staging.insert(FilePath::from_path("./src/hello"), ObjHash::new(b"hello"));

        io.write(staging.clone()).unwrap();
        let red = io.read().unwrap().unwrap();
        assert_eq!(red, staging);
    }
}


