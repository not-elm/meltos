pub mod branch;
pub mod error;
pub mod file_system;
pub mod object;
pub mod io;
pub mod operation;

#[cfg(feature = "cli")]
pub mod command;
mod remote_client;


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
