use crate::branch::BranchIo;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::work_branch::WorkBranchIo;

pub mod branch;
pub mod error;
pub mod file_system;
pub mod object;
pub mod io;
pub mod operation;

pub struct RepositoryIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    branch: BranchIo<Fs, Io>,
    work_branch: WorkBranchIo<Fs, Io>,
    fs: Fs,
}

impl<Fs, Io> RepositoryIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub fn open(fs: Fs) -> error::Result<RepositoryIo<Fs, Io>> {
        let work_branch = WorkBranchIo(FsIo::new(fs.clone()));
        let work_branch_name = work_branch.read()?;

        let branch = BranchIo::new(work_branch_name, fs.clone());
        branch.unpack_project()?;

        Ok(Self {
            work_branch,
            branch,
            fs,
        })
    }
}

#[cfg(test)]
mod tests {
    // use crate::branch::BranchName;
    // use crate::file_system::FileSystem;
    // use crate::file_system::mock::MockFileSystem;
    // use crate::RepositoryIo;
    //

    //
    // #[test]
    // fn unpack_workspace_files() {
    //     let mock = MockFileSystem::default();
    //     let p1 = "./hello.txt";
    //     let p2 = "./src/sample";
    //     mock.write_all(p1, b"hello").unwrap();
    //     mock.write_all(p2, b"sample").unwrap();
    //
    //     let io = RepositoryIo::init(mock.clone()).unwrap();
    //     // file_system.branch.stage(".").unwrap();
    //     todo!();
    //     // io.branch.commit("commit").unwrap();
    //     // mock.delete(p1).unwrap();
    //     // mock.delete(p2).unwrap();
    //     //
    //     // RepositoryIo::open(mock.clone()).unwrap();
    //     // assert_eq!(mock.try_read_to_end(p1).unwrap(), b"hello");
    //     // assert_eq!(mock.try_read_to_end(p2).unwrap(), b"sample");
    // }
}
