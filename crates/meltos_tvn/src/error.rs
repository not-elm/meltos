use std::num::ParseIntError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::branch::BranchName;
use crate::file_system::FilePath;
use crate::object::commit::CommitHash;
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

    #[error("obj hash buffer is invalid")]
    ObjHashBufferIsInvalid,

    #[error("obj type is invalid : hash={0}")]
    InvalidObjBuffer(ObjHash),

    #[error("expect obj type is {0} bad was {1}")]
    InvalidObjType(String, String),

    #[error("commit obj buffer is invalid")]
    CommitObjBufferIsInValid,

    #[error("not found object: hash={0}")]
    NotfoundObj(ObjHash),

    #[error("not found stages")]
    NotfoundStages,

    #[error("not found workspace file : path={0}")]
    NotfoundWorkspaceFile(FilePath),

    #[error("changed file does not exists")]
    ChangedFileNotExits,

    #[error("not found trace: commit hash={0}")]
    NotfoundTrace(CommitHash),

    #[error("not found local commits")]
    NotfoundLocalCommits,

    #[error("not found head: branch={0}")]
    NotfoundHead(BranchName),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error(transparent)]
    FromUtf8(#[from] FromUtf8Error),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),

    #[error("expect obj type is file or delete bad was.")]
    InvalidWorkspaceObj,

    #[error("failed connect server : detail={0}")]
    FailedConnectServer(String),
}

