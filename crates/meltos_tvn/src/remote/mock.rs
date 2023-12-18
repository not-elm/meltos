use async_trait::async_trait;

use meltos::room::RoomId;
use meltos_util::sync::arc_mutex::ArcMutex;

use crate::branch::BranchName;
use crate::file_system::mock::MockFileSystem;
use crate::io::bundle::{Bundle, BundleIo};
use crate::operation::push::PushParam;
use crate::remote::{CommitFetchable, CommitPushable};

#[derive(Debug, Default, Clone)]
pub struct MockRemoteClient {
    pub push_param: ArcMutex<Option<PushParam>>,
    pub fs: MockFileSystem,
}


impl MockRemoteClient {
    #[allow(unused)]
    pub fn new(fs: MockFileSystem) -> Self {
        Self {
            fs,
            push_param: Default::default(),
        }
    }
}

#[async_trait]
impl CommitPushable for MockRemoteClient {
    type Error = std::io::Error;

    async fn push(&mut self, param: PushParam) -> std::io::Result<()> {
        *self.push_param.lock().await = Some(param);
        Ok(())
    }
}

#[async_trait]
impl CommitFetchable for MockRemoteClient {
    type Error = std::io::Error;

    async fn fetch(
        &mut self,
        _target_branch: Option<BranchName>,
    ) -> std::result::Result<Bundle, Self::Error> {
        let bundle = BundleIo::new(self.fs.clone()).create().unwrap();
        Ok(bundle)
    }
}
