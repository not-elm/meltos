use std::collections::VecDeque;
use std::str::FromStr;

use auto_delegate::Delegate;

use meltos_util::macros::{Deref, Display};

use crate::error;
use crate::io::atomic::head::CommitText;
use crate::object::{AsMeta, Decodable, Encodable, ObjHash, ObjMeta};

#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Display, Delegate, Deref, Hash)]
#[to(Encodable)]
pub struct CommitHash(pub ObjHash);

impl Decodable for CommitHash {
    #[inline]
    fn decode(buf: &[u8]) -> error::Result<Self> {
        Ok(Self(ObjHash::decode(buf)?))
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommitObj {
    pub parents: Vec<CommitHash>,
    pub text: CommitText,
    pub committed_objs_tree: ObjHash,
}


impl AsMeta for CommitObj {
    #[inline]
    fn as_meta(&self) -> error::Result<ObjMeta> {
        Ok(ObjMeta::compress(self.encode()?)?)
    }
}


impl TryFrom<ObjMeta> for CommitObj {
    type Error = error::Error;

    #[inline]
    fn try_from(meta: ObjMeta) -> Result<Self, Self::Error> {
        CommitObj::decode(&meta.buf)
    }
}


static HEAD: &[u8] = b"COMMIT";

impl Encodable for CommitObj {
    fn encode(&self) -> error::Result<Vec<u8>> {
        let mut buf = HEAD.to_vec();
        let parents_count = self.parents.len();
        buf.extend(format!("{parents_count}\0").as_bytes());
        for hash in &self.parents {
            buf.extend(hash.encode()?);
            buf.push(b'\0');
        }

        buf.extend(self.committed_objs_tree.encode()?);
        buf.push(b'\0');

        buf.extend(self.text.as_bytes());

        Ok(buf)
    }
}


impl Decodable for CommitObj {
    fn decode(buf: &[u8]) -> error::Result<Self> {
        let mut buf = buf[HEAD.len()..]
            .split(|b| b == &b'\0')
            .collect::<VecDeque<&[u8]>>();
        let parents = decode_parents(&mut buf)?;
        let committed_objs_tree = ObjHash::decode(buf.pop_front().unwrap())?;
        let text = CommitText::decode(buf.pop_front().unwrap())?;
        Ok(Self {
            parents,
            committed_objs_tree,
            text,
        })
    }
}

fn decode_parents(buf: &mut VecDeque<&[u8]>) -> error::Result<Vec<CommitHash>> {
    let count_buf = std::str::from_utf8(buf.pop_front().unwrap())?;
    let parents_count = usize::from_str(count_buf)?;
    let mut parents = Vec::with_capacity(parents_count);
    for _ in 0..parents_count {
        parents.push(CommitHash::decode(buf.pop_front().unwrap())?);
    }

    Ok(parents)
}


#[cfg(test)]
mod tests {
    use crate::io::atomic::head::CommitText;
    use crate::object::{Decodable, Encodable, ObjHash};
    use crate::object::commit::{CommitHash, CommitObj, HEAD};

    #[test]
    fn serialize() {
        let hash1 = CommitHash(ObjHash::new(b"hello"));
        let hash2 = CommitHash(ObjHash::new(b"world!"));
        let parents = vec![hash1.clone(), hash2.clone()];
        let commit_text = CommitText::from("commit");
        let tree = ObjHash::new(b"hash");
        let commit = CommitObj {
            parents: parents.clone(),
            text: commit_text.clone(),
            committed_objs_tree: tree.clone(),
        };
        let buf = commit.encode().unwrap();
        let h = HEAD.len();
        assert_eq!(&buf[..h], HEAD);
        assert_eq!(&buf[h..h + 2], b"2\0");
        assert_eq!(&buf[h + 2..h + 2 + 40], hash1.encode().unwrap());
        assert_eq!(&buf[h + 2 + 41..h + 2 + 41 + 40], hash2.encode().unwrap());
    }


    #[test]
    fn decode() {
        let hash1 = CommitHash(ObjHash::new(b"hello"));
        let hash2 = CommitHash(ObjHash::new(b"world!"));
        let parents = vec![hash1.clone(), hash2.clone()];
        let commit_text = CommitText::from("commit");
        let tree = ObjHash::new(b"hash");
        let commit = CommitObj {
            parents: parents.clone(),
            text: commit_text.clone(),
            committed_objs_tree: tree.clone(),
        };
        let buf = commit.encode().unwrap();
        let decoded = CommitObj::decode(&buf).unwrap();
        assert_eq!(decoded, commit);
    }
}