use crate::branch::structs::branch_name::BranchName;
use crate::error;

#[async_trait::async_trait]
pub trait LocalBranchIo {
    async fn branch_names(&self) -> error::Result<Vec<BranchName>>;


    async fn fetch_by(&self, branch_name: &BranchName) -> error::Result<BranchName>;
}
