use thiserror::Error;

use crate::branch::BranchName;
use crate::object::ObjectHash;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("repository already initialized")]
    RepositoryAlreadyInitialized,

    #[error("branch {0} has been already initialized")]
    BranchAlreadyInitialized(BranchName),

    #[error("not found object: hash={0}")]
    NotfoundObj(ObjectHash),

    #[error("not found stages")]
    NotfoundStages,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
