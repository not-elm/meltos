use thiserror::Error;
use meltos::discussion::id::DiscussionId;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("user id not exists")]
    UserIdNotExists,

    #[error("discussion not exists id: {0}")]
    DiscussionNotExists(DiscussionId),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}


unsafe impl Send for Error{}