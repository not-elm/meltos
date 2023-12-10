use serde::Serialize;

pub trait AsBinary {
    fn as_binary(&self) -> bincode::Result<Vec<u8>>;
}


impl<S: Serialize> AsBinary for S {
    fn as_binary(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }
}
