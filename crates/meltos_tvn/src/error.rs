use thiserror::Error;

use crate::branch::BranchName;
use crate::object::ObjHash;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("repository already initialized")]
    RepositoryAlreadyInitialized,

    #[error("branch {0} has been already initialized")]
    BranchAlreadyInitialized(BranchName),

    #[error("obj hash is empty")]
    ObjHashIsEmpty,

    #[error("not found object: hash={0}")]
    NotfoundObj(ObjHash),

    #[error("not found stages")]
    NotfoundStages,

    #[error("not found local commits")]
    NotfoundLocalCommits,

    #[error("not found head")]
    NotfoundHead,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
