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
    use crate::file_system::mock::MockFileSystem;
    use crate::object::commit::CommitHash;
    use crate::operation::init;

    pub(crate) fn init_owner_branch(mock: MockFileSystem) -> CommitHash {
        init::Init::new(mock).execute(&BranchName::owner()).unwrap()
    }
}
