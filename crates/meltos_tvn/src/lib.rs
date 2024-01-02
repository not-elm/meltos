pub mod branch;
pub mod error;
pub mod file_system;
pub mod io;
pub mod object;
pub mod operation;

pub mod encode;

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::object::commit::CommitHash;
    use crate::operation::init;

    pub(crate) fn init_main_branch(mock: MockFileSystem) -> CommitHash {
        init::Init::new(BranchName::owner(), mock).execute().unwrap()
    }
}
