use thiserror::Error;

use crate::branch::BranchName;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("repository already initialized")]
    RepositoryAlreadyInitialized,

    #[error("branch {0} has been already initialized")]
    BranchAlreadyInitialized(BranchName),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}