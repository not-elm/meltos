use std::collections::HashSet;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::{CommitText, HeadIo};
use crate::io::atomic::local_commits::LocalCommitsIo;
use crate::io::atomic::object::ObjIo;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::commit::CommitObj;
use crate::object::local_commits::LocalCommitsObj;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    head: HeadIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    local_commits: LocalCommitsIo<Fs, Io>,
    trace_tree: TraceTreeIo<Fs, Io>,
}


impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, fs: Fs) -> CommitObjIo<Fs, Io> {
        CommitObjIo {
            head: HeadIo::new(branch_name.clone(), fs.clone()),
            object: ObjIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name.clone(), fs.clone()),
            trace_tree: TraceTreeIo::new(branch_name, fs),
        }
    }
}

impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn read_local_commits(&self) -> error::Result<Vec<CommitObj>> {
        let Some(LocalCommitsObj(local_hashes)) = self.local_commits.read()?
            else {
                return Ok(vec![]);
            };
        let mut commit_objs = Vec::with_capacity(local_hashes.len());
        for hash in local_hashes {
            commit_objs.push(self.object.read_to_commit(&hash)?);
        }
        Ok(commit_objs)
    }


    pub fn read_head(&self) -> error::Result<Option<CommitObj>> {
        let Some(hash) = self.head.read()?
            else {
                return Ok(None);
            };
        Ok(Some(self.read(&hash)?))
    }


    pub fn read(&self, commit_hash: &ObjHash) -> error::Result<CommitObj> {
        let commit_obj = self.object.try_read_obj(commit_hash)?;
        CommitObj::try_from(commit_obj)
    }


    pub fn create(
        &self,
        commit_text: impl Into<CommitText>,
        staging_hash: ObjHash,
    ) -> error::Result<CommitObj> {
        let head_commit = self.head.read()?;
        Ok(CommitObj {
            parents: head_commit.map(|hash| vec![hash]).unwrap_or_default(),
            text: commit_text.into(),
            committed_objs_tree: staging_hash,
        })
    }


    #[inline]
    pub fn reset_local_commits(&self) -> error::Result {
        self.local_commits.write(&LocalCommitsObj::default())
    }

    pub fn read_obj_hashes_associate_with(&self, commit_hash: ObjHash) -> error::Result<HashSet<ObjHash>> {
        let mut obj_hashes = HashSet::new();
        self.read_commit_objs(commit_hash, &mut obj_hashes)?;
        match self.trace_tree.read()? {
            Some(tree) => {
                for t in tree.0.into_values() {
                    obj_hashes.insert(t);
                }
                Ok(obj_hashes)
            }
            None => Ok(obj_hashes)
        }
    }

    fn read_commit_objs(&self, commit_hash: ObjHash, obj_hashes: &mut HashSet<ObjHash>) -> error::Result {
        let commit_obj = self.read(&commit_hash)?;
        self.read_objs_with_in_tree(&commit_obj, obj_hashes)?;
        obj_hashes.insert(commit_hash);
        for parent_commit_hash in commit_obj.parents.into_iter() {
            self.read_commit_objs(parent_commit_hash, obj_hashes)?;
        }

        Ok(())
    }

    fn read_objs_with_in_tree(&self, commit_obj: &CommitObj, obj_hashes: &mut HashSet<ObjHash>) -> error::Result {
        let tree = self
            .object
            .read_to_tree(&commit_obj.committed_objs_tree)?;
        obj_hashes.insert(commit_obj.committed_objs_tree.clone());
        for hash in tree.0.into_values() {
            obj_hashes.insert(hash);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::branch::BranchName;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::commit_obj::CommitObjIo;
    use crate::io::trace_tree::TraceTreeIo;
    use crate::object::ObjHash;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;

    #[test]
    fn local_commits_is_empty_if_not_committed() {
        let local_commit_objs = CommitObjIo::new(BranchName::main(), MockFileSystem::default())
            .read_local_commits()
            .unwrap();
        assert_eq!(local_commit_objs, vec![]);
    }


    #[test]
    fn local_commit_count_is_2() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let commit_obj = CommitObjIo::new(branch, mock);

        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        let local_commits = commit_obj.read_local_commits().unwrap();
        assert_eq!(local_commits.len(), 2);
    }


    #[test]
    fn read_objs_associated_with_all_commits() {
        let mock = MockFileSystem::default();
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let trace = TraceTreeIo::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let obj = ObjIo::new(mock.clone());
        let commit_obj = CommitObjIo::new(branch, mock.clone());

        mock.write("./hello/hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash1 = commit.execute("commit text").unwrap();

        mock.write("./src/sample", b"sample").unwrap();
        mock.write("./t", b"t").unwrap();
        stage.execute(".").unwrap();
        let commit_hash2 = commit.execute("commit text").unwrap();

        let mut objs = commit_obj
            .read_obj_hashes_associate_with(commit_hash2.clone()).unwrap()
            .into_iter()
            .collect::<Vec<ObjHash>>();
        objs.sort();
        let trace_obj = trace.read().unwrap().unwrap();
        let mut expect = vec![
            commit_hash1.clone(),
            commit_hash2.clone(),
            ObjHash::new(b"hello"),
            ObjHash::new(b"sample"),
            ObjHash::new(b"t"),
        ];
        expect.push(obj.read_to_commit(&commit_hash1).unwrap().committed_objs_tree);
        expect.push(obj.read_to_commit(&commit_hash2).unwrap().committed_objs_tree);
        for (_, obj) in trace_obj.iter() {
            expect.push(obj.clone());
        }

        let mut expect = expect.into_iter().collect::<HashSet<ObjHash>>()
            .into_iter()
            .collect::<Vec<ObjHash>>();
        expect.sort();
        assert_eq!(objs, expect);
    }
}