use std::io;
use crate::branch::BranchName;

use crate::error;
use crate::file_system::FileSystem;
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
}


impl<Fs, Io> UnZip<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: io::Read + io::Write
{
    pub fn new(branch_name: BranchName, fs: Fs) -> UnZip<Fs, Io>{
        Self{
            workspace: WorkspaceIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name, fs)
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
        let Some(trace_tree) = self.trace_tree.read()? else {
            return Err(error::Error::NotfoundTrace);
        };

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
    use crate::operation::init::Init;
    use crate::operation::stage::Stage;
    use crate::operation::unzip::UnZip;

    #[test]
    fn failed_if_not_exists_trace() {
        let mock = MockFileSystem::default();
        let unzip = UnZip::new(BranchName::main(), mock);
        match unzip.execute(){
            Err(error::Error::NotfoundTrace) => {},
            _ => panic!("expected return error::Error::NotfoundTrace bad was")
        }
    }


    #[test]
    fn success_if_committed() -> error::Result{
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let init = Init::new(branch.clone(), mock.clone());
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let unzip = UnZip::new(branch, mock.clone());

        init.execute()?;
        mock.write("hello", b"hello")?;
        stage.execute("hello")?;
        commit.execute("commit text")?;
        mock.delete("hello")?;

        unzip.execute()?;
        assert_eq!(mock.try_read("hello")?, b"hello");
        Ok(())
    }
}