use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use meltos_util::macros::{Deref, DerefMut};

use crate::io::{FilePath, OpenIo, TvnIo};
use crate::object::ObjectHash;

#[derive(Debug, Clone)]
pub struct TreeIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: TvnIo<Open, Io>,
    file_path: FilePath,
}


impl<Open, Io> TreeIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new(file_path: impl Into<FilePath>, io: TvnIo<Open, Io>) -> TreeIo<Open, Io> {
        Self {
            io,
            file_path: file_path.into(),
        }
    }


    pub fn write_tree(&self, tree: &Tree) -> std::io::Result<()> {
        self.io
            .write(&self.file_path, &serde_json::to_vec(&tree)?)?;
        Ok(())
    }

    pub fn read_tree(&self) -> std::io::Result<Option<Tree>> {
        let Some(json) = self.io.read_to_end(&self.file_path)? else {
            return Ok(None);
        };

        Ok(serde_json::from_slice::<Tree>(&json).ok())
    }


    pub fn reset(&self) -> std::io::Result<()> {
        self.io
            .write(&self.file_path, &serde_json::to_vec(&Tree::default())?)?;
        Ok(())
    }


    pub fn read_object_hash(&self, file_path: &FilePath) -> std::io::Result<Option<ObjectHash>> {
        let Some(tree) = self.read_tree()? else {
            return Ok(None);
        };
        Ok(tree.0.get(file_path).cloned())
    }

    pub fn write_object_hash(
        &self,
        target_path: FilePath,
        object_hash: ObjectHash,
    ) -> std::io::Result<()> {
        let mut tree = self.read_tree()?.unwrap_or_default();
        tree.0.insert(target_path, object_hash);
        self.io
            .write(&self.file_path, &serde_json::to_vec(&tree)?)?;
        Ok(())
    }
}


#[derive(Serialize, Deserialize, Default, Clone, Deref, DerefMut, Debug, Eq, PartialEq)]
pub struct Tree(HashMap<FilePath, ObjectHash>);


impl Tree {
    pub fn changed_hash(&self, path: &FilePath, hash: &ObjectHash) -> bool {
        if let Some(old_hash) = self.0.get(path) {
            old_hash != hash
        } else {
            true
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::io::mock::MockOpenIo;
    use crate::io::{FilePath, TvnIo};
    use crate::object::ObjectHash;
    use crate::tree::{Tree, TreeIo};

    #[test]
    fn stage_json() {
        let mut stage = Tree::default();
        stage
            .0
            .insert(FilePath::from("hello"), ObjectHash("world".to_string()));
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(
            json,
            json!({
                "hello": "world"
            })
            .to_string()
        );
    }


    #[test]
    fn none_if_none_wrote() {
        let io = TreeIo::new("stage", TvnIo::new(MockOpenIo::default()));
        let hash = io.read_object_hash(&FilePath::from("hello")).unwrap();
        assert!(hash.is_none());
    }

    #[test]
    fn some_hash_if_wrote() {
        let io = TreeIo::new("stage", TvnIo::new(MockOpenIo::default()));
        let path = FilePath::from("hello");
        io.write_object_hash(path.clone(), ObjectHash("hash".to_string()))
            .unwrap();
        let hash = io.read_object_hash(&path).unwrap();
        assert_eq!(hash, Some(ObjectHash("hash".to_string())));
    }
}
