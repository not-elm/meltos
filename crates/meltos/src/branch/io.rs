use crate::branch::structs::branch_name::BranchName;

#[async_trait::async_trait]
pub trait LocalBranchIo {
    async fn branch_names(&self) -> Vec<BranchName>;


    async fn checkout(&self, branch_name: &BranchName);


    async fn commit(&self);
}
