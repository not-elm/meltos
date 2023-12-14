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
    pub fn init(open: Open) -> std::io::Result<RepositoryIo<Open, Io>> {
        let io = BranchIo::new_main(open);
        io.init()?;
        Ok(RepositoryIo {
            io
        })
    }


}