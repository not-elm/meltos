use meltos_core::error::MelResult;

pub mod gz;


pub trait CompressionBuf: Send + Sync {
    fn encode(&self, buf: &[u8]) -> MelResult<Vec<u8>>;

    fn decode(&self, buf: &[u8]) -> MelResult<Vec<u8>>;

    fn file_extension() -> Option<&'static str>;
}
