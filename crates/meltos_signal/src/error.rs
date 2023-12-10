use thiserror::Error;
use tokio::task::JoinError;

pub type Result<T = ()> = std::result::Result<T, Error>;


#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Join(#[from] JoinError),
}
