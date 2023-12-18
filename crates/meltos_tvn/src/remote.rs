use async_trait::async_trait;

use crate::branch::BranchName;
use crate::io::bundle::Bundle;
use crate::operation::push::PushParam;

pub(crate) mod mock;

#[async_trait]
pub trait CommitPushable: Send + Sync {
    /// Sends the currently locally committed data to the remote.
    ///
    /// This function called when the `push` command is executed.
    async fn push(&mut self, param: PushParam) -> std::io::Result<()>;
}


#[async_trait]
pub trait CommitFetchable: Send + Sync {
    /// Fetch commits from server.
    ///
    /// if any branch name is specified in `target_branch`,  only the data associated with tha branch is retrieved;
    /// otherwise, data from all branches is retrieved.
    ///
    async fn fetch(&mut self, target_branch: Option<BranchName>) -> std::io::Result<Bundle>;
}

