use crate::error;
use meltos_util::compression::CompressionBuf;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct BlockFileBuf(Vec<u8>);


impl BlockFileBuf {
    pub fn new<C: CompressionBuf + Default>(file_buf: &[u8]) -> error::Result<Self> {
        Ok(Self(C::default().decode(file_buf)?))
    }
}


#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct BlockHash(String);


impl BlockHash {
    pub fn new(file_buf: &[u8]) -> Self {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(file_buf);

        Self(hasher.digest().to_string())
    }
}


pub fn generate<C: CompressionBuf + Default>(
    file_buf: &[u8],
) -> error::Result<(BlockHash, BlockFileBuf)> {
    let buf = BlockFileBuf::new::<C>(file_buf)?;
    let hash = BlockHash::new(file_buf);
    Ok((hash, buf))
}


#[cfg(test)]
mod tests {
    use crate::block::file::BlockHash;

    #[test]
    fn hash_length() {
        let hash1 = BlockHash::new(b"hello");
        let hash2 = BlockHash::new(b"hello world\ntest");
        assert_eq!(hash1.0.len(), 40);
        assert_eq!(hash2.0.len(), 40);
    }
}
