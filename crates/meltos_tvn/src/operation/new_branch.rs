use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::atomic::workspace::WorkspaceIo;

#[derive(Debug, Clone)]
pub struct NewBranch<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    branch_name: BranchName,
    workspace: WorkspaceIo<Fs, Io>,
    trace: TraceIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
}


impl<Fs, Io> NewBranch<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{

    ///
    ///
    /// * copy `HEAD FILE` from old branch
    pub fn execute(&self, _from: BranchName, _to: BranchName) -> error::Result{
        todo!()
    }
}