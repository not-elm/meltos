pub mod gz;

pub trait CompressionBuf: Send + Sync {
    fn zip(&self, buf: &[u8]) -> std::io::Result<Vec<u8>>;

    fn unzip(&self, buf: &[u8]) -> std::io::Result<Vec<u8>>;

    fn file_extension() -> Option<&'static str>;
}
