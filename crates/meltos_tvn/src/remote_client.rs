pub(crate) mod mock;

use async_trait::async_trait;
use crate::operation::push::PushParam;

#[async_trait]
pub trait CommitSendable: Send + Sync{
    /// Sends the currently locally committed data to the remote.
    ///
    /// This function called when the `push` command is executed.
    async fn send(&self, param: PushParam) -> std::io::Result<()>;
}


