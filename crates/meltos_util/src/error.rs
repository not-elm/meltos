use std::fmt::Debug;

use log::debug;

pub type Result<T = ()> = std::result::Result<T, Error>;


pub type Error = Box<dyn std::error::Error>;


pub trait LogIfError {
    /// output error log if failed.
    fn log_if_error(self);
}


impl<T, E> LogIfError for std::result::Result<T, E>
where
    E: Debug,
{
    fn log_if_error(self) {
        if let Err(error) = self {
            debug!("{error:?}");
        }
    }
}
