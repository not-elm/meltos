use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::file_system::std_fs::StdFileSystem;
use crate::io::atomic::local_commits::LocalCommitsIo;
use crate::io::atomic::work_branch::WorkingIo;
use crate::io::bundle::BundleIo;
use crate::operation::checkout::Checkout;
use crate::operation::commit::Commit;
use crate::operation::init::Init;
use crate::operation::merge::Merge;
use crate::operation::patch::Patch;
use crate::operation::push::Push;
use crate::operation::save::Save;
use crate::operation::stage::Stage;
use crate::operation::unzip::UnZip;

pub mod checkout;
pub mod commit;
pub mod init;
pub mod merge;
pub mod new_branch;
pub mod patch;
pub mod push;
pub mod save;
pub mod stage;
pub mod unzip;

#[derive(Debug)]
pub struct Operations<Fs = StdFileSystem>
    where
        Fs: FileSystem + Clone,
{
    pub init: Init<Fs>,
    pub patch: Patch<Fs>,
    pub stage: Stage<Fs>,
    pub commit: Commit<Fs>,
    pub push: Push<Fs>,
    pub save: Save<Fs>,
    pub bundle: BundleIo<Fs>,
    pub checkout: Checkout<Fs>,
    pub unzip: UnZip<Fs>,
    pub merge: Merge<Fs>,
    pub local_commits: LocalCommitsIo<Fs>,
    pub working: WorkingIo<Fs>,
    fs: Fs,
    branch_name: BranchName,
}

impl<Fs> Operations<Fs>
    where
        Fs: FileSystem + Clone,
{
    #[inline]
    pub fn new_main(fs: Fs) -> Operations<Fs> {
        Self::new(BranchName::owner(), fs)
    }

    #[inline]
    pub fn new_work(fs: Fs) -> error::Result<Operations<Fs>> {
        let work = WorkingIo::new(fs.clone());
        Ok(Self::new(work.try_read()?, fs))
    }

    pub fn new(branch_name: BranchName, fs: Fs) -> Operations<Fs> {
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
            merge: Merge::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name.clone(), fs.clone()),
            working: WorkingIo::new(fs.clone()),
            fs,
            branch_name,
        }
    }
}

impl<Fs> Clone for Operations<Fs>
    where
        Fs: FileSystem + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.branch_name.clone(), self.fs.clone())
    }
}
