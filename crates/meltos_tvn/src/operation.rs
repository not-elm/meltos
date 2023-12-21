use std::fs::File;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::file::StdFileSystem;
use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;
use crate::io::bundle::BundleIo;
use crate::operation::checkout::Checkout;
use crate::operation::commit::Commit;
use crate::operation::init::Init;
use crate::operation::patch::Patch;
use crate::operation::push::Push;
use crate::operation::save::Save;
use crate::operation::stage::Stage;
use crate::operation::unzip::UnZip;

pub mod checkout;
pub mod commit;
pub mod init;
pub mod new_branch;
pub mod patch;
pub mod push;
pub mod save;
pub mod stage;
pub mod unzip;
pub mod merge;


#[derive(Debug)]
pub struct Operations<Fs = StdFileSystem, Io = File>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub init: Init<Fs, Io>,
    pub patch: Patch<Fs, Io>,
    pub stage: Stage<Fs, Io>,
    pub commit: Commit<Fs, Io>,
    pub push: Push<Fs, Io>,
    pub save: Save<Fs, Io>,
    pub bundle: BundleIo<Fs, Io>,
    pub checkout: Checkout<Fs, Io>,
    pub unzip: UnZip<Fs, Io>,
    fs: Fs,
    branch_name: BranchName,
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
        Ok(Self::new(work.try_read()?, fs))
    }

    pub fn new(branch_name: BranchName, fs: Fs) -> Operations<Fs, Io> {
        Self {
            init: Init::new(branch_name.clone(), fs.clone()),
            patch: Patch::new(fs.clone()),
            stage: Stage::new(branch_name.clone(), fs.clone()),
            commit: Commit::new(branch_name.clone(), fs.clone()),
            push: Push::new(branch_name.clone(), fs.clone()),
            save: Save::new(fs.clone()),
            bundle: BundleIo::new(fs.clone()),
            checkout: Checkout::new(fs.clone()),
            unzip: UnZip::new(fs.clone()),
            fs,
            branch_name,
        }
    }
}


impl<Fs, Io> Clone for Operations<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.branch_name.clone(), self.fs.clone())
    }
}
