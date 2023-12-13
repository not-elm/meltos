use serde::{Deserialize, Serialize};
use crate::branch::BranchName;
use crate::io::{OpenIo, TvcIo};

use crate::object::ObjectHash;
use crate::tree::Tree;


#[derive(Debug, Clone)]
pub struct CommitIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    io: TvcIo<Open, Io>,
    branch_name: BranchName
}


impl<Open, Io> CommitIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, open: Open) -> CommitIo<Open, Io>{
        Self{
            branch_name,
            io: TvcIo::new(open)
        }
    }

    pub fn commit(&self, stage: Tree) -> std::io::Result<()>{
        let meta = self.create_commit(stage)?;
        self.io.write(&format!(".meltos/commits/{}", meta.hash), &serde_json::to_vec(&meta.commit)?)?;
        self.io.write(&format!(".meltos/branches/{}/HEAD", self.branch_name), meta.hash.0.as_bytes())?;
        Ok(())
    }


    fn create_commit(&self, stage: Tree) -> std::io::Result<CommitMeta>{
        let head_commit = self.head_commit_hash()?;
        let commit = Commit{
            parent: head_commit,
            stage
        };
        Ok(CommitMeta::new(commit))
    }


    pub fn head_commit_hash(&self) -> std::io::Result<Option<ObjectHash>>{
        let buf = self.io.read_to_end(&format!(".meltos/branches/{}/HEAD", self.branch_name))?;
        Ok(buf.and_then(|buf|Some(ObjectHash(String::from_utf8(buf).ok()?))))
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
