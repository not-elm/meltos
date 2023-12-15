use meltos_util::macros::{Deref, DerefMut};
use crate::object::ObjHash;


#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Deref, DerefMut, Default)]
pub struct LocalCommitsObj(pub Vec<ObjHash>);


impl LocalCommitsObj {
    pub fn new(buf: Vec<u8>) -> Self {
        Self(String::from_utf8(buf)
            .unwrap()
            .split('\0')
            .filter(|hash|!hash.is_empty())
            .map(|hash| ObjHash(hash.to_string()))
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
    use crate::object::local_commits::LocalCommitsObj;
    use crate::object::ObjHash;

    #[test]
    fn serialize_and_deserialize(){
        let hash = ObjHash::new(b"hello");
        let hash2 = ObjHash::new(b"world");
        let local = LocalCommitsObj(vec![hash.clone(), hash2.clone()]);

        let buf = local.to_buf();
        assert_eq!(local, LocalCommitsObj::new(buf));
    }
}