pub mod branch;
pub mod error;
pub mod file_system;
pub mod io;
pub mod object;
pub mod operation;

#[cfg(feature = "cli")]
pub mod command;
pub mod encode;
pub mod remote_client;


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::object::commit::CommitHash;
    use crate::operation::init;

    pub(crate) fn init_main_branch(mock: MockFileSystem) -> CommitHash {
        init::Init::new(BranchName::main(), mock).execute().unwrap()
    }
}
