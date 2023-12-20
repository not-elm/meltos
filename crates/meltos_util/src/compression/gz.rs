use flate2::bufread::{GzDecoder, GzEncoder};
use std::io::Read;

use crate::compression::CompressionBuf;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash, Default)]
pub struct Gz;

impl CompressionBuf for Gz {
    fn zip(&self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut gz = GzEncoder::new(buf, flate2::Compression::default());
        let mut buffer = Vec::new();
        gz.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn unzip(&self, buf: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut gz = GzDecoder::new(buf);
        let mut buffer = Vec::new();
        gz.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn file_extension() -> Option<&'static str> {
        Some("gz")
    }
}

#[cfg(test)]
mod tests {

    use crate::compression::gz::Gz;
    use crate::compression::CompressionBuf;

    #[test]
    fn ascii() {
        let buff = b"hello world!";
        let encode = Gz.zip(buff).unwrap();
        let decode = Gz.unzip(&encode).unwrap();

        assert_eq!(decode, buff);
    }

    #[test]
    fn japanese() {
        let buff = "日本語".as_bytes();
        let encode = Gz.zip(buff).unwrap();
        let decode = Gz.unzip(&encode).unwrap();

        assert_eq!(decode, buff);
    }
}
