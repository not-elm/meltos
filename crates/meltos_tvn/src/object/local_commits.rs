use meltos_util::macros::{Deref, DerefMut};
use crate::object::ObjectHash;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deref, DerefMut)]
pub struct LocalCommits(pub Vec<ObjectHash>);


impl LocalCommits {
    pub fn new(buf: Vec<u8>) -> Self {
        Self(String::from_utf8(buf)
            .unwrap()
            .split('\0')
            .map(|hash| ObjectHash(hash.to_string()))
            .collect())
    }


    pub fn to_buf(&self) -> Vec<u8>{
        self
            .0
            .iter()
            .map(|hash|hash.to_string())
            .collect::<Vec<String>>()
            .join("\0")
            .into_bytes()
    }
}




#[cfg(test)]
mod tests{
    use crate::object::local_commits::LocalCommits;
    use crate::object::ObjectHash;

    #[test]
    fn serialize_and_deserialize(){
        let hash = ObjectHash::new(b"hello");
        let hash2 = ObjectHash::new(b"world");
        let local = LocalCommits(vec![hash.clone(), hash2.clone()]);

        let buf = local.to_buf();
        assert_eq!(local, LocalCommits::new(buf));
    }
}