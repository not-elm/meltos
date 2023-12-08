use crate::thread::ThreadId;
use thiserror::Error;


pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("thread not exists id = {0:?}")]
    ThreadNotExists(ThreadId),
}
