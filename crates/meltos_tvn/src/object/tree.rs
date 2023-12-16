use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use meltos_util::macros::{Deref, DerefMut};

use crate::error;
use crate::file_system::{FilePath, FileSystem, FsIo};
use crate::object::{Obj, ObjHash};

#[derive(Debug, Clone)]
pub struct TreeIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
    file_path: FilePath,
}

impl<Fs, Io> TreeIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new(file_path: impl Into<FilePath>, io: FsIo<Fs, Io>) -> TreeIo<Fs, Io> {
        Self {
            io,
            file_path: file_path.into(),
        }
    }

    pub fn write_tree(&self, tree: &TreeObj) -> std::io::Result<()> {
        self.io
            .write(&self.file_path, &serde_json::to_vec(&tree)?)?;
        Ok(())
    }

    pub fn read(&self) -> std::io::Result<Option<TreeObj>> {
        let Some(json) = self.io.read(&self.file_path)? else {
            return Ok(None);
        };

        Ok(serde_json::from_slice::<TreeObj>(&json).ok())
    }

    pub fn reset(&self) -> std::io::Result<()> {
        self.io
            .write(&self.file_path, &serde_json::to_vec(&TreeObj::default())?)?;
        Ok(())
    }

    pub fn read_object_hash(&self, file_path: &FilePath) -> std::io::Result<Option<ObjHash>> {
        let Some(tree) = self.read()? else {
            return Ok(None);
        };
        Ok(tree.0.get(file_path).cloned())
    }

    pub fn write_object_hash(
        &self,
        target_path: FilePath,
        object_hash: ObjHash,
    ) -> std::io::Result<()> {
        let mut tree = self.read()?.unwrap_or_default();
        tree.0.insert(target_path, object_hash);
        self.io.write(&self.file_path, &serde_json::to_vec(&tree)?)?;
        Ok(())
    }
}


#[repr(transparent)]
#[derive(Serialize, Deserialize, Default, Clone, Deref, DerefMut, Debug, Eq, PartialEq)]
pub struct TreeObj(pub HashMap<FilePath, ObjHash>);

impl TreeObj {
    pub fn changed_hash(&self, path: &FilePath, hash: &ObjHash) -> bool {
        if let Some(old_hash) = self.0.get(path) {
            old_hash != hash
        } else {
            true
        }
    }

    #[inline]
    pub fn as_obj(&self) -> error::Result<Obj> {
        let buf = serde_json::to_vec(self)?;
        Ok(Obj::compress(buf)?)
    }


    pub fn replace_by(&mut self, tree: TreeObj) {
        for (file_path, hash) in tree.0.into_iter() {
            self.0.insert(file_path, hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::file_system::FilePath;
    use crate::object::ObjHash;
    use crate::object::tree::TreeObj;

    #[test]
    fn stage_json() {
        let mut stage = TreeObj::default();
        stage
            .0
            .insert(FilePath::from("hello"), ObjHash("world".to_string()));
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(
            json,
            json!({
                "hello": "world"
            })
                .to_string()
        );
    }
}
