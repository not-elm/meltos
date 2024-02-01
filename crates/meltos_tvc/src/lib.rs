pub mod branch;
pub mod error;
pub mod file_system;
pub mod io;
pub mod object;
pub mod operation;

pub mod encode;
mod time;

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::object::commit::CommitHash;
    use crate::operation::init;

    pub(crate) async fn init_owner_branch(mock: MemoryFileSystem) -> CommitHash {
        init::Init::new(mock)
            .execute(&BranchName::owner())
            .await
            .unwrap()
    }
}
