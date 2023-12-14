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
}


impl<Open, Io> RepositoryIo<Open, Io>
where
    Open: OpenIo<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn init(open: Open) -> error::Result<RepositoryIo<Open, Io>> {
        if !open.all_file_path(".")?.is_empty() {
            return Err(error::Error::RepositoryAlreadyInitialized);
        }

        let branch = BranchIo::new_main(open.clone());
        branch.init()?;

        let work_branch = WorkBranchIo(TvnIo::new(open));
        work_branch.write(&BranchName::main())?;

        Ok(RepositoryIo {
            branch,
            work_branch,
        })
    }


    pub fn new_branch(&self, from: &BranchName, to: &BranchName) -> error::Result {
        self.work_branch.write(to)?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::io::mock::MockOpenIo;
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
}
