use meltos::room::RoomId;
use meltos_tvc::file_system::FileSystem;
use meltos_tvc::io::atomic::object::ObjIo;
use meltos_tvc::io::bundle::{Bundle, BundleIo};
use meltos_tvc::operation::save::Save;

use crate::tvc::file_system::BackendFileSystem;

mod file_system;


#[derive(Debug, Clone)]
pub struct TvcBackendIo<Fs: FileSystem + Clone> {
    bundle: BundleIo<BackendFileSystem<Fs>>,
    save: Save<BackendFileSystem<Fs>>,
    obj: ObjIo<BackendFileSystem<Fs>>
}


impl<Fs: FileSystem + Clone> TvcBackendIo<Fs> {
    pub fn new(room_id: RoomId, fs: Fs) -> TvcBackendIo<Fs> {
        let fs = BackendFileSystem::new(room_id, fs);
        Self {
            bundle: BundleIo::new(fs.clone()),
            save: Save::new(fs.clone()),
            obj: ObjIo::new(fs)
        }
    }


    #[inline(always)]
    pub fn total_objs_size(&self) -> meltos_tvc::error::Result<usize>{
        self.obj.total_objs_size()
    }


    #[inline(always)]
    pub fn save(&self, bundle: Bundle) -> meltos_tvc::error::Result {
        self.save.execute(bundle)
    }

    #[inline(always)]
    pub fn bundle(&self) -> meltos_tvc::error::Result<Bundle> {
        self.bundle.create()
    }
}


