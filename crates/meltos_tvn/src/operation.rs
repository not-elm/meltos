use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;
use crate::operation::commit::Commit;
use crate::operation::init::Init;
use crate::operation::push::Push;
use crate::operation::stage::Stage;

pub mod init;
pub mod stage;
pub mod commit;
pub mod push;
pub mod unzip;
mod new_branch;


#[derive(Debug, Clone)]
pub struct Operations<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub init: Init<Fs, Io>,
    pub stage: Stage<Fs, Io>,
    pub commit: Commit<Fs, Io>,
    pub push: Push<Fs, Io>,
}


impl<Fs, Io> Operations<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new_main(fs: Fs) -> Operations<Fs, Io> {
        Self::new(BranchName::main(), fs)
    }

    #[inline]
    pub fn new_work(fs: Fs) -> error::Result<Operations<Fs, Io>> {
        let work = WorkingIo::new(fs.clone());
        Ok(Self::new(work.read()?, fs))
    }

    pub fn new(branch_name: BranchName, fs: Fs) -> Operations<Fs, Io> {
        Self {
            init: Init::new(branch_name.clone(), fs.clone()),
            stage: Stage::new(branch_name.clone(), fs.clone()),
            commit: Commit::new(branch_name.clone(), fs.clone()),
            push: Push::new(branch_name, fs),
        }
    }
}

