use thiserror::Error;

pub type MelResult<T = ()> = std::result::Result<T, MelError>;


#[derive(Error, Debug)]
pub enum MelError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    BinCode(#[from] bincode::Error),
}
