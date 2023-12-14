use serde::{Deserialize, Serialize};
use crate::atomic::head::CommitText;
use crate::object::ObjectHash;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommitMeta {
    pub hash: ObjectHash,
    pub commit: Commit,
}

impl CommitMeta {
    //   pub fn create_commit(
    //     &self,
    //     commit_text: impl Into<CommitText>,
    //     stage: ObjectHash,
    // ) -> std::file_system::Result<CommitMeta> {
    //     let head_commit = self.head_commit_hash()?;
    //     let commit = Commit {
    //         parent: head_commit,
    //         text: commit_text.into(),
    //         stage,
    //     };
    //     Ok(CommitMeta::new(commit))
    // }

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
    pub text: CommitText,
    pub stage: ObjectHash,
}
