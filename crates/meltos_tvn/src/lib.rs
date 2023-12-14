use crate::branch::{BranchIo, BranchName};
use crate::io::{OpenIo, TvnIo};
use crate::work_branch::WorkBranchIo;

pub mod branch;
pub mod commit;
pub mod error;
pub mod io;
pub mod now;
pub mod object;
pub mod stage;
pub mod tree;
pub mod work_branch;
pub mod workspace;

pub struct RepositoryIo<Open, Io>
where
    Open: OpenIo<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    branch: BranchIo<Open, Io>,
    work_branch: WorkBranchIo<Open, Io>,
    open: Open,
}

impl<Open, Io> RepositoryIo<Open, Io>
where
    Open: OpenIo<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn init(open: Open) -> error::Result<RepositoryIo<Open, Io>> {
        if !open.all_file_path(".meltos")?.is_empty() {
            return Err(error::Error::RepositoryAlreadyInitialized);
        }

        let branch = BranchIo::new_main(open.clone());
        branch.init()?;

        let work_branch = WorkBranchIo(TvnIo::new(open.clone()));
        work_branch.write(&BranchName::main())?;

        Ok(RepositoryIo {
            branch,
            work_branch,
            open,
        })
    }

    pub fn open(open: Open) -> error::Result<RepositoryIo<Open, Io>> {
        let work_branch = WorkBranchIo(TvnIo::new(open.clone()));
        let work_branch_name = work_branch.read()?;

        let branch = BranchIo::new(work_branch_name, open.clone());
        branch.unpack_project()?;

        Ok(Self {
            work_branch,
            branch,
            open,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::io::mock::MockOpenIo;
    use crate::io::OpenIo;
    use crate::RepositoryIo;

    #[test]
    fn create_work_after_initialized() {
        let mock = MockOpenIo::default();
        let io = RepositoryIo::init(mock.clone()).unwrap();
        assert_eq!(io.work_branch.read().unwrap(), BranchName::main());
    }

    #[test]
    fn error_if_already_initialized() {
        let mock = MockOpenIo::default();
        RepositoryIo::init(mock.clone()).unwrap();
        assert!(RepositoryIo::init(mock.clone()).is_err());
    }

    #[test]
    fn unpack_workspace_files() {
        let mock = MockOpenIo::default();
        let p1 = "./hello.txt";
        let p2 = "./src/sample";
        mock.write(p1, b"hello").unwrap();
        mock.write(p2, b"sample").unwrap();

        let io = RepositoryIo::init(mock.clone()).unwrap();
        io.branch.stage(".").unwrap();
        io.branch.commit("commit").unwrap();
        mock.delete(p1).unwrap();
        mock.delete(p2).unwrap();

        RepositoryIo::open(mock.clone()).unwrap();
        assert_eq!(mock.try_read_to_end(p1).unwrap(), b"hello");
        assert_eq!(mock.try_read_to_end(p2).unwrap(), b"sample");
    }
}
