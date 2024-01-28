use meltos::room::RoomId;
use meltos::user::UserId;
use meltos_tvc::branch::BranchName;
use meltos_tvc::file_system::FileSystem;
use meltos_tvc::io::atomic::head::HeadIo;
use meltos_tvc::io::atomic::object::ObjIo;
use meltos_tvc::io::bundle::{Bundle, BundleIo};
use meltos_tvc::operation::save::Save;

use crate::tvc::file_system::BackendFileSystem;

mod file_system;

#[derive(Debug, Clone)]
pub struct TvcBackendIo<Fs: FileSystem + Clone> {
    bundle: BundleIo<BackendFileSystem<Fs>>,
    save: Save<BackendFileSystem<Fs>>,
    obj: ObjIo<BackendFileSystem<Fs>>,
    head: HeadIo<BackendFileSystem<Fs>>,
}

impl<Fs: FileSystem + Clone> TvcBackendIo<Fs> {
    pub fn new(room_id: RoomId, fs: Fs) -> TvcBackendIo<Fs> {
        let fs = BackendFileSystem::new(room_id, fs);
        Self {
            bundle: BundleIo::new(fs.clone()),
            save: Save::new(fs.clone()),
            obj: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs),
        }
    }

    #[inline(always)]
    pub async fn total_objs_size(&self) -> meltos_tvc::error::Result<usize> {
        self.obj.total_objs_size().await
    }

    #[inline(always)]
    pub async fn save(&self, bundle: Bundle) -> meltos_tvc::error::Result {
        self.save.execute(bundle).await
    }

    #[inline(always)]
    pub async fn write_head(&self, branch_name: &BranchName) -> meltos_tvc::error::Result {
        if let Some(owner_head) = self.head.read(&BranchName::owner()).await? {
            self.head.write(branch_name, &owner_head).await?;
        }
        Ok(())
    }

    #[inline(always)]
    pub async fn leave(&self, user_id: UserId) -> meltos_tvc::error::Result {
        self.head.delete(&BranchName(user_id.0)).await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn bundle(&self) -> meltos_tvc::error::Result<Bundle> {
        self.bundle.create().await
    }
}
