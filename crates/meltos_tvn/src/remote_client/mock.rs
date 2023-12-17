use async_trait::async_trait;

use meltos_util::sync::arc_mutex::ArcMutex;

use crate::operation::push::PushParam;
use crate::remote_client::CommitSendable;

#[derive(Debug, Default)]
pub struct MockRemoteClient {
    pub push_param: ArcMutex<Option<PushParam>>,
}


#[async_trait]
impl CommitSendable for MockRemoteClient {
    async fn send(&mut self, param: PushParam) -> std::io::Result<()> {
        *self.push_param.lock().await = Some(param);
        Ok(())
    }
}
