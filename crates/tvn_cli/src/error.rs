use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tvn(#[from] meltos_tvn::error::Error),

    #[error(transparent)]
    Client(#[from] meltos_client::error::Error),

    #[error("please login")]
    SessionIdNotExists,
}
