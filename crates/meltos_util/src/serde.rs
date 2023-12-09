use crate::error;
use serde::Serialize;

pub trait AsBinary {
    fn as_binary(&self) -> error::Result<Vec<u8>>;
}


impl<S: Serialize> AsBinary for S {
    fn as_binary(&self) -> error::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
}
