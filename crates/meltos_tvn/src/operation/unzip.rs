use std::io;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::io::trace_tree::TraceTreeIo;

pub struct UnZip<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write
{
    workspace: WorkspaceIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
}


impl<Fs, Io> UnZip<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: io::Read + io::Write
{
    pub fn new(branch_name: BranchName, fs: Fs) -> UnZip<Fs, Io> {
        Self {
            workspace: WorkspaceIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(branch_name, fs.clone()),
            trace_tree: TraceTreeIo::new(fs),
        }
    }
}


impl<Fs, Io> UnZip<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write
{
    /// Restore committed data into the workspace.
    pub fn execute(&self) -> error::Result {
        let head = self.head.read()?;
        let trace_tree = self.trace_tree.read(&head)?;
        for (path, hash) in trace_tree.iter() {
            self.workspace.unpack(path, &self.object.try_read_obj(hash)?.buf)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::error;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::operation::unzip::UnZip;
    use crate::tests::init_main_branch;

    #[test]
    fn success_if_committed() -> error::Result {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::main();

        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let unzip = UnZip::new(branch, mock.clone());

        mock.write("hello", b"hello")?;
        stage.execute("hello")?;
        commit.execute("commit text")?;
        mock.delete("hello")?;

        unzip.execute()?;
        assert_eq!(mock.try_read("hello")?, b"hello");
        Ok(())
    }
}