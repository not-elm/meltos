use crate::file_system::std_fs::StdFileSystem;
use crate::file_system::FileSystem;
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
use crate::operation::un_stage::UnStage;
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
pub mod un_stage;
pub mod unzip;

#[derive(Debug)]
pub struct Operations<Fs = StdFileSystem>
where
    Fs: FileSystem + Clone,
{
    pub init: Init<Fs>,
    pub patch: Patch<Fs>,
    pub stage: Stage<Fs>,
    pub un_stage: UnStage<Fs>,
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
}

impl<Fs> Operations<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Operations<Fs> {
        Self {
            init: Init::new(fs.clone()),
            patch: Patch::new(fs.clone()),
            stage: Stage::new(fs.clone()),
            un_stage: UnStage::new(fs.clone()),
            commit: Commit::new(fs.clone()),
            push: Push::new(fs.clone()),
            save: Save::new(fs.clone()),
            bundle: BundleIo::new(fs.clone()),
            checkout: Checkout::new(fs.clone()),
            unzip: UnZip::new(fs.clone()),
            merge: Merge::new(fs.clone()),
            local_commits: LocalCommitsIo::new(fs.clone()),
            working: WorkingIo::new(fs.clone()),
            fs,
        }
    }
}

impl<Fs> Clone for Operations<Fs>
where
    Fs: FileSystem + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.fs.clone())
    }
}
