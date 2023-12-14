use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;
use crate::branch::BranchName;
use crate::error;

use crate::file_system::{FileSystem, FsIo};
use crate::object::ObjectHash;

#[derive(Debug, Clone)]
pub struct HeadIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    io: FsIo<Fs, Io>,
    branch_name: BranchName,
}

impl<Fs, Io> HeadIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> HeadIo<Fs, Io> {
        Self {
            branch_name,
            io: FsIo::new(fs),
        }
    }

    pub fn write_head(
        &self,
        commit_hash: ObjectHash,
    ) -> std::io::Result<()> {
        self.io.write_all(
            &format!(".meltos/branches/{}/HEAD", self.branch_name),
            &commit_hash.serialize_to_buf(),
        )?;
        Ok(())
    }

    pub fn head_commit_hash(&self) -> error::Result<Option<ObjectHash>> {
        let Some(buf) = self
            .io
            .read_to_end(&format!(".meltos/branches/{}/HEAD", self.branch_name))?
            else {
                return Ok(None);
            };

        Ok(Some(ObjectHash::from_serialized_buf(&buf)?))
    }
}


#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct CommitText(pub String);
impl_string_new_type!(CommitText);

#[cfg(test)]
mod tests {
    // use crate::io::atomic::commit::{Commit, CommitText, HeadIo};
    // use crate::io::atomic::object::ObjectHash;
    // use crate::branch::BranchName;
    // use crate::file_system::FilePath;
    // use crate::file_system::mock::MockFsIo;
    // use crate::tree::Tree;
    //
    // #[test]
    // fn create_head_and_commit_obj() {
    //     let mut stage_tree = Tree::default();
    //     stage_tree.insert(FilePath::from("hello"), ObjectHash::new(b"hello"));
    //     let obj = stage_tree.as_obj().unwrap();
    //     let mock = MockFsIo::default();
    //     let file_system = HeadIo::new(BranchName::main(), mock.clone());
    //     file_system.write_head(obj.hash.clone()).unwrap();
    //     let head = file_system.head_commit_hash().unwrap().unwrap();
    //     let commit = file_system.read_commit(&head).unwrap();
    //     assert_eq!(
    //         commit,
    //         Some(Commit {
    //             parent: None,
    //             stage: obj.hash,
    //             text: CommitText::from("commit"),
    //         })
    //     );
    // }
    //
    // #[test]
    // fn attach_parent() {
    //     let mut stage_tree = Tree::default();
    //     stage_tree.insert(FilePath::from("hello"), ObjectHash::new(b"hello"));
    //
    //     let mock = MockFsIo::default();
    //     let file_system = HeadIo::new(BranchName::main(), mock.clone());
    //     file_system.write_head("commit1", ObjectHash::new(b"hello1")).unwrap();
    //     let first_commit = file_system.head_commit_hash().unwrap().unwrap();
    //     let mut stage_tree2 = Tree::default();
    //     stage_tree2.insert(FilePath::from("commit2"), ObjectHash::new(b"commit2"));
    //     file_system.write_head("commit2", ObjectHash::new(b"hello2")).unwrap();
    //
    //     let second_commit = file_system.head_commit_hash().unwrap().unwrap();
    //     let commit = file_system.read_commit(&second_commit).unwrap();
    //     assert_eq!(
    //         commit,
    //         Some(Commit {
    //             parent: Some(first_commit),
    //             stage: ObjectHash::new(b"hello2"),
    //             text: CommitText::from("commit2"),
    //         })
    //     );
    // }
}
