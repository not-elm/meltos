pub mod gz;


pub trait CompressionBuf: Send + Sync {

    fn encode(&self, buf: &[u8]) -> std::io::Result<Vec<u8>>;

    fn decode(&self, buf: &[u8]) -> std::io::Result<Vec<u8>>;

    fn file_extension() -> Option<&'static str>;
}
