use serde::{Deserialize, Serialize};

use crate::branch::BranchName;
use crate::io::{OpenIo, TvcIo};
use crate::object::ObjectHash;
use crate::tree::Tree;

#[derive(Debug, Clone)]
pub struct CommitIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    io: TvcIo<Open, Io>,
    branch_name: BranchName,
}


impl<Open, Io> CommitIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, open: Open) -> CommitIo<Open, Io> {
        Self {
            branch_name,
            io: TvcIo::new(open),
        }
    }

    pub fn commit(&self, stage: Tree) -> std::io::Result<()> {
        let meta = self.create_commit(stage)?;
        self.io.write(
            &format!(".meltos/commits/{}", meta.hash),
            &serde_json::to_vec(&meta.commit)?,
        )?;
        self.io.write(
            &format!(".meltos/branches/{}/HEAD", self.branch_name),
            meta.hash.0.as_bytes(),
        )?;
        Ok(())
    }

    pub fn head_commit_hash(&self) -> std::io::Result<Option<ObjectHash>> {
        let Some(buf) = self
            .io
            .read_to_end(&format!(".meltos/branches/{}/HEAD", self.branch_name))?
        else {
            return Ok(None);
        };

        Ok(Some(ObjectHash(String::from_utf8(buf).unwrap())))
    }


    pub fn read_commit(&self, commit_hash: &ObjectHash) -> std::io::Result<Option<Commit>> {
        let Some(buf) = self
            .io
            .read_to_end(&format!(".meltos/commits/{commit_hash}"))?
        else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_slice(&buf)?))
    }

    fn create_commit(&self, stage: Tree) -> std::io::Result<CommitMeta> {
        let head_commit = self.head_commit_hash()?;
        let commit = Commit {
            parent: head_commit,
            stage,
        };
        Ok(CommitMeta::new(commit))
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommitMeta {
    pub hash: ObjectHash,
    pub commit: Commit,
}


impl CommitMeta {
    pub fn new(commit: Commit) -> Self {
        Self {
            hash: ObjectHash::new(&serde_json::to_vec(&commit).unwrap()),
            commit,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Commit {
    pub parent: Option<ObjectHash>,
    pub stage: Tree,
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::commit::{Commit, CommitIo};
    use crate::io::mock::MockOpenIo;
    use crate::io::FilePath;
    use crate::object::ObjectHash;
    use crate::tree::Tree;

    #[test]
    fn create_head_and_commit_obj() {
        let mut stage_tree = Tree::default();
        stage_tree.insert(FilePath::from("hello"), ObjectHash::new(b"hello"));

        let mock = MockOpenIo::default();
        let io = CommitIo::new(BranchName::main(), mock.clone());
        io.commit(stage_tree.clone()).unwrap();
        let head = io.head_commit_hash().unwrap().unwrap();
        let commit = io.read_commit(&head).unwrap();
        assert_eq!(
            commit,
            Some(Commit {
                parent: None,
                stage: stage_tree,
            })
        );
    }


    #[test]
    fn attach_parent() {
        let mut stage_tree = Tree::default();
        stage_tree.insert(FilePath::from("hello"), ObjectHash::new(b"hello"));

        let mock = MockOpenIo::default();
        let io = CommitIo::new(BranchName::main(), mock.clone());
        io.commit(stage_tree.clone()).unwrap();
        let first_commit = io.head_commit_hash().unwrap().unwrap();
        let mut stage_tree2 = Tree::default();
        stage_tree2.insert(FilePath::from("commit2"), ObjectHash::new(b"commit2"));
        io.commit(stage_tree2.clone()).unwrap();

        let second_commit = io.head_commit_hash().unwrap().unwrap();
        let commit = io.read_commit(&second_commit).unwrap();
        assert_eq!(
            commit,
            Some(Commit {
                parent: Some(first_commit),
                stage: stage_tree2,
            })
        );
    }
}
