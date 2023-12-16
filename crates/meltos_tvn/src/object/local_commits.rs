use std::collections::VecDeque;
use std::str::FromStr;

use meltos_util::macros::{Deref, DerefMut};

use crate::object::{AsMeta, Decodable, Encodable, ObjMeta};
use crate::object::commit::CommitHash;

#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deref, DerefMut, Default)]
pub struct LocalCommitsObj(pub Vec<CommitHash>);


impl AsMeta for LocalCommitsObj {
    #[inline]
    fn as_meta(&self) -> crate::error::Result<ObjMeta> {
        Ok(ObjMeta::compress(self.encode()?)?)
    }
}


static HEADER: &[u8] = b"LOCAL_COMMITS\0";

impl Encodable for LocalCommitsObj {
    fn encode(&self) -> crate::error::Result<Vec<u8>> {
        let mut buf = HEADER.to_vec();
        buf.extend(format!("{}\0", self.0.len()).as_bytes());
        buf.extend(self
            .0
            .iter()
            .map(|hash| hash.to_string())
            .collect::<Vec<String>>()
            .join("\0")
            .as_bytes()
        );
        Ok(buf)
    }
}


impl Decodable for LocalCommitsObj {
    fn decode(buf: &[u8]) -> crate::error::Result<Self> {
        let mut buf = buf[HEADER.len()..]
            .split(|b| b == &b'\0')
            .collect::<VecDeque<&[u8]>>();
        let hash_count = buf.pop_front().unwrap();
        let hash_count = usize::from_str(std::str::from_utf8(hash_count)?)?;

        let mut hashes = Vec::with_capacity(hash_count);
        for _ in 0..hash_count {
            hashes.push(CommitHash::decode(buf.pop_front().unwrap())?);
        }
        Ok(Self(hashes))
    }
}


#[cfg(test)]
mod tests {
    use crate::object::{Decodable, Encodable, ObjHash};
    use crate::object::commit::CommitHash;
    use crate::object::local_commits::LocalCommitsObj;

    #[test]
    fn encode() {
        let hash = CommitHash(ObjHash::new(b"hash"));
        let commits = LocalCommitsObj(vec![hash.clone()]);
        let buf = commits.encode().unwrap();
        let l = b"LOCAL_COMMITS\0".len();
        assert_eq!(&buf[..l], b"LOCAL_COMMITS\0");
        assert_eq!(&buf[l..l + 2], b"1\0");
        assert_eq!(&buf[l + 2..], hash.to_string().as_bytes());
    }


    #[test]
    fn decode() {
        let hash1 = CommitHash(ObjHash::new(b"hash1"));
        let hash2 = CommitHash(ObjHash::new(b"hash2"));
        let commits = LocalCommitsObj(vec![hash1, hash2]);
        let buf = commits.encode().unwrap();
        let decoded = LocalCommitsObj::decode(&buf).unwrap();
        assert_eq!(commits, decoded);
    }
}