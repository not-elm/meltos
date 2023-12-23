use thiserror::Error;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Tvn(#[from] meltos_tvn::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
