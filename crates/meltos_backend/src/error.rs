use thiserror::Error;
use meltos::discussion::id::DiscussionId;
use meltos::room::RoomId;

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

    #[error("database already removed room_id: {0}")]
    DatabaseAlreadyRemoved(RoomId)
}


unsafe impl Send for Error{}