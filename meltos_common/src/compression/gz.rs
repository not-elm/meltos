use std::io::Read;

use flate2::bufread::GzEncoder;
use flate2::read::GzDecoder;

use crate::compression::CompressionBuf;
use crate::error::MelResult;

pub struct Gz;


impl CompressionBuf for Gz {
    fn encode(&self, buf: &[u8]) -> MelResult<Vec<u8>> {
        let mut gz = GzEncoder::new(buf, flate2::Compression::default());
        let mut buffer = Vec::new();
        gz.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn decode(&self, buf: &[u8]) -> MelResult<Vec<u8>> {
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
        let encode = Gz.encode(buff).unwrap();
        let decode = Gz.decode(&encode).unwrap();

        assert_eq!(decode, buff);
    }


    #[test]
    fn japanese() {
        let buff = "日本語".as_bytes();
        let encode = Gz.encode(buff).unwrap();
        let decode = Gz.decode(&encode).unwrap();

        assert_eq!(decode, buff);
    }
}
