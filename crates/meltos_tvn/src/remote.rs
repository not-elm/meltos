use async_trait::async_trait;

use crate::branch::BranchName;
use crate::io::bundle::Bundle;
use crate::operation::push::PushParam;

pub mod local;
pub(crate) mod mock;

#[async_trait]
pub trait CommitPushable: Send + Sync {
    type Error: std::error::Error;


    /// Sends the currently locally committed data to the remote.
    ///
    /// This function called when the `push` command is executed.
    async fn push(&mut self, param: PushParam) -> std::result::Result<(), Self::Error>;
}


#[async_trait]
pub trait CommitFetchable: Send + Sync {
    type Error: std::error::Error;


    /// Fetch commits from server.
    ///
    /// if any branch name is specified in `target_branch`,  only the data associated with tha branch is retrieved;
    /// otherwise, data from all branches is retrieved.
    ///
    async fn fetch(
        &mut self,
        target_branch: Option<BranchName>,
    ) -> std::result::Result<Bundle, Self::Error>;
}
