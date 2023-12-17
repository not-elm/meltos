pub(crate) mod mock;

use crate::operation::push::PushParam;
use async_trait::async_trait;

#[async_trait]
pub trait CommitSendable: Send + Sync {
    /// Sends the currently locally committed data to the remote.
    ///
    /// This function called when the `push` command is executed.
    async fn send(&mut self, param: PushParam) -> std::io::Result<()>;
}
