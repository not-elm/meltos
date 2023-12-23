use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

use meltos_util::macros::{Deref, DerefMut};

use crate::file_system::{FilePath, FileSystem, };
use crate::object::{AsMeta, Decodable, Encodable, ObjHash, ObjMeta};
use crate::{error, impl_serialize_and_deserialize};

#[derive(Debug, Clone)]
pub struct TreeIo<Fs>
where
    Fs: FileSystem
{
    fs: Fs,
    file_path: FilePath,
}

impl<Fs> TreeIo<Fs>
where
    Fs: FileSystem
{
    #[inline]
    pub fn new(file_path: impl Into<FilePath>, fs: Fs) -> TreeIo<Fs> {
        Self {
            fs,
            file_path: file_path.into(),
        }
    }

    pub fn write_tree(&self, tree: &TreeObj) -> error::Result<()> {
        self.fs.write(&self.file_path, &tree.encode()?)?;
        Ok(())
    }

    pub fn read(&self) -> error::Result<Option<TreeObj>> {
        let Some(buf) = self.fs.read(&self.file_path)? else {
            return Ok(None);
        };

        Ok(Some(TreeObj::decode(&buf)?))
    }

    pub fn reset(&self) -> error::Result<()> {
        self.fs
            .write(&self.file_path, &TreeObj::default().encode()?)?;
        Ok(())
    }

    pub fn read_object_hash(&self, file_path: &FilePath) -> error::Result<Option<ObjHash>> {
        let Some(tree) = self.read()? else {
            return Ok(None);
        };
        Ok(tree.0.get(file_path).cloned())
    }

    pub fn write_object_hash(
        &self,
        target_path: FilePath,
        object_hash: ObjHash,
    ) -> error::Result<()> {
        let mut tree = self.read()?.unwrap_or_default();
        tree.0.insert(target_path, object_hash);
        self.fs.write(&self.file_path, &tree.encode()?)?;
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Default, Clone, Deref, DerefMut, Debug, Eq, PartialEq)]
pub struct TreeObj(pub HashMap<FilePath, ObjHash>);
impl_serialize_and_deserialize!(TreeObj);

impl TreeObj {
    pub const HEADER: &'static [u8] = b"TREE\0";

    pub fn changed_hash(&self, path: &FilePath, hash: &ObjHash) -> bool {
        if let Some(old_hash) = self.0.get(path) {
            old_hash != hash
        } else {
            true
        }
    }

    pub fn replace_by(&mut self, tree: TreeObj) {
        for (file_path, hash) in tree.0.into_iter() {
            self.0.insert(file_path, hash);
        }
    }
}

impl AsMeta for TreeObj {
    fn as_meta(&self) -> error::Result<ObjMeta> {
        let buf = self.encode()?;
        Ok(ObjMeta::compress(buf)?)
    }
}

impl Encodable for TreeObj {
    fn encode(&self) -> error::Result<Vec<u8>> {
        #[derive(Ord, PartialOrd, Eq, PartialEq)]
        struct KeyValue<'a> {
            file_path: &'a FilePath,
            hash: &'a ObjHash,
        }

        let mut key_value = self
            .0
            .iter()
            .map(|(file_path, hash)| {
                KeyValue {
                    file_path,
                    hash,
                }
            })
            .collect::<Vec<KeyValue>>();
        key_value.sort();

        let mut buf = TreeObj::HEADER.to_vec();
        buf.extend(format!("{}\0", key_value.len()).as_bytes());
        for k_v in key_value {
            buf.extend(format!("{}\0{}\0", k_v.file_path, k_v.hash).as_bytes());
        }

        Ok(buf)
    }
}

impl Decodable for TreeObj {
    fn decode(buf: &[u8]) -> error::Result<Self> {
        let mut buf = buf[TreeObj::HEADER.len()..]
            .split(|b| b == &b'\0')
            .collect::<VecDeque<&[u8]>>();

        let entry_count = decode_entry_count(&mut buf)?;
        let mut tree = TreeObj(HashMap::with_capacity(entry_count));
        for _ in 0..entry_count {
            let entry_buf = buf.pop_front().unwrap();
            let file_path = FilePath::from_path(String::from_utf8(entry_buf.to_vec())?);
            let hash = ObjHash::decode(buf.pop_front().unwrap())?;
            tree.0.insert(file_path, hash);
        }

        Ok(tree)
    }
}

fn decode_entry_count(buf: &mut VecDeque<&[u8]>) -> error::Result<usize> {
    let entry_count_buf = buf.pop_front().unwrap();
    let entry_count_str = std::str::from_utf8(entry_count_buf)?;
    Ok(usize::from_str(entry_count_str)?)
}

#[cfg(test)]
mod tests {
    use crate::file_system::FilePath;
    use crate::object::tree::TreeObj;
    use crate::object::{Decodable, Encodable, ObjHash};

    #[test]
    fn encode() {
        let mut tree = TreeObj::default();
        let p1 = FilePath::from_path("hello");
        let h1 = ObjHash::new(b"hello");
        let p2 = FilePath::from_path("./src/sample.txt");
        let h2 = ObjHash::new(b"sample");
        tree.0.insert(p1.clone(), h1.clone());
        tree.0.insert(p2.clone(), h2.clone());

        let buf = tree.encode().unwrap();
        let h = TreeObj::HEADER.len();
        assert_eq!(&buf[..h], TreeObj::HEADER);
        assert_eq!(&buf[h..h + 2], b"2\0");

        let p2_len = p2.len();

        let i = h + 2 + 1 + p2_len + 41;
        assert_eq!(&buf[h + 2..i], format!("{p2}\0{h2}\0").as_bytes());
        assert_eq!(&buf[i..], format!("{p1}\0{h1}\0").as_bytes());
    }

    #[test]
    fn decode() {
        let mut tree = TreeObj::default();
        let p1 = FilePath::from_path("hello");
        let h1 = ObjHash::new(b"hello");
        let p2 = FilePath::from_path("./src/sample.txt");
        let h2 = ObjHash::new(b"sample");
        tree.0.insert(p1.clone(), h1.clone());
        tree.0.insert(p2.clone(), h2.clone());

        let buf = tree.encode().unwrap();
        let decoded = TreeObj::decode(&buf).unwrap();
        assert_eq!(decoded, tree);
    }

    #[test]
    fn deserialize() {
        let mut tree = TreeObj::default();
        let p1 = FilePath::from_path("hello");
        let h1 = ObjHash::new(b"hello");
        tree.0.insert(p1.clone(), h1.clone());

        let json = serde_json::to_string(&tree).unwrap();
        let decoded: TreeObj = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, tree);
    }
}
