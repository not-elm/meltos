use crate::branch::BranchIo;
use crate::io::OpenIo;

pub mod branch;
pub mod commit;
pub mod io;
pub mod now;
pub mod object;
pub mod stage;
pub mod tree;
pub mod workspace;
mod error;


pub struct RepositoryIo<Open, Io>
    where
        Open: OpenIo<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    io: BranchIo<Open, Io>,
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
        let io = BranchIo::new_main(open);
        io.init()?;
        Ok(RepositoryIo {
            io
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::io::mock::MockOpenIo;
    use crate::RepositoryIo;

    #[test]
    fn error_if_already_initialized() {
        let mock = MockOpenIo::default();
        RepositoryIo::init(mock.clone()).unwrap();
        assert!(RepositoryIo::init(mock.clone()).is_err());
    }
}