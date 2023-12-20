use std::collections::HashSet;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::{CommitText, HeadIo};
use crate::io::atomic::local_commits::LocalCommitsIo;
use crate::io::atomic::object::ObjIo;
use crate::io::bundle::BundleObject;
use crate::io::trace_tree::TraceTreeIo;
use crate::object::commit::{CommitHash, CommitObj};
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
    branch_name: BranchName,
}


impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> CommitObjIo<Fs, Io> {
        CommitObjIo {
            head: HeadIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name.clone(), fs.clone()),
            trace_tree: TraceTreeIo::new(fs),
            branch_name,
        }
    }
}

impl<Fs, Io> CommitObjIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn read_local_commits(&self) -> error::Result<Vec<CommitObj>> {
        let Some(LocalCommitsObj(local_hashes)) = self.local_commits.read()? else {
            return Ok(Vec::with_capacity(0));
        };

        let mut commit_objs = Vec::with_capacity(local_hashes.len());
        for hash in local_hashes {
            commit_objs.push(self.object.read_to_commit(&hash)?);
        }
        Ok(commit_objs)
    }


    pub fn read_head(&self) -> error::Result<CommitObj> {
        let hash = self.head.try_read(&self.branch_name)?;
        self.read(&hash)
    }


    pub fn read(&self, commit_hash: &ObjHash) -> error::Result<CommitObj> {
        let commit = self.object.try_read_obj(commit_hash)?;
        commit.commit()
    }


    pub fn create(
        &self,
        commit_text: impl Into<CommitText>,
        staging_hash: ObjHash,
    ) -> error::Result<CommitObj> {
        let head_commit = self.head.read(&self.branch_name)?;
        let parents = head_commit
            .map(|head| vec![head])
            .unwrap_or(Vec::with_capacity(0));
        Ok(CommitObj {
            parents,
            text: commit_text.into(),
            committed_objs_tree: staging_hash,
        })
    }


    #[inline]
    pub fn reset_local_commits(&self) -> error::Result {
        self.local_commits.write(&LocalCommitsObj::default())
    }

    pub fn read_objs_associated_with_local(&self) -> error::Result<Vec<BundleObject>> {
        let Some(local_commits) = self.local_commits.read()? else {
            return Err(error::Error::NotfoundLocalCommits);
        };
        let mut obj_hashes = HashSet::new();
        for commit_hash in local_commits.0 {
            self.read_obj_hashes_associate_with(&mut obj_hashes, commit_hash, false)?;
        }

        let mut obj_bufs = Vec::with_capacity(obj_hashes.len());
        for hash in obj_hashes {
            let Some(compressed_buf) = self.object.read(&hash)? else {
                return Err(error::Error::NotfoundObj(hash));
            };
            obj_bufs.push(BundleObject {
                hash,
                compressed_buf,
            });
        }
        Ok(obj_bufs)
    }

    pub fn read_obj_hashes_associate_with(
        &self,
        obj_hashes: &mut HashSet<ObjHash>,
        commit_hash: CommitHash,
        read_parents: bool
    ) -> error::Result {
        let tree = self.trace_tree.read(&commit_hash)?;
        self.read_commit_objs(commit_hash, obj_hashes, read_parents)?;
        for obj_hash in tree.0.into_values() {
            obj_hashes.insert(obj_hash);
        }
        Ok(())
    }

    fn read_commit_objs(
        &self,
        commit_hash: CommitHash,
        obj_hashes: &mut HashSet<ObjHash>,
        read_parents: bool
    ) -> error::Result {
        let commit_obj = self.read(&commit_hash)?;
        self.read_objs_with_in_tree(&commit_obj, obj_hashes, read_parents)?;
        obj_hashes.insert(commit_hash.0);

        Ok(())
    }

    fn read_objs_with_in_tree(
        &self,
        commit_obj: &CommitObj,
        obj_hashes: &mut HashSet<ObjHash>,
        read_parents: bool
    ) -> error::Result {
        let tree = self.object.read_to_tree(&commit_obj.committed_objs_tree)?;
        obj_hashes.insert(commit_obj.committed_objs_tree.clone());
        for hash in tree.0.into_values() {
            obj_hashes.insert(hash);
        }
        if read_parents{
            for hash in commit_obj.parents.iter(){
                self.read_obj_hashes_associate_with(obj_hashes, hash.clone(), read_parents)?;
            }
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
    use crate::tests::init_main_branch;

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
        init_main_branch(mock.clone());
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let commit_obj = CommitObjIo::new(branch, mock.clone());

        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        mock.write("./hello.txt", b"hello2").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        let local_commits = commit_obj.read_local_commits().unwrap();
        assert_eq!(local_commits.len(), 3);
    }


    #[test]
    fn read_objs_associated_with_all_commits() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());
        let branch = BranchName::main();
        let stage = Stage::new(branch.clone(), mock.clone());
        let trace = TraceTreeIo::new(mock.clone());
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
        let mut objs =  HashSet::new();
        commit_obj.read_obj_hashes_associate_with(&mut objs, commit_hash2.clone(), true).unwrap();
        let mut objs = objs.into_iter().collect::<Vec<ObjHash>>();
        objs.sort();
        let trace_obj = trace.read(&commit_hash2).unwrap();
        let mut expect = vec![
            null_commit_hash.clone().0,
            commit_hash1.clone().0,
            commit_hash2.clone().0,
            ObjHash::new(b"FILE\0hello"),
            ObjHash::new(b"FILE\0sample"),
            ObjHash::new(b"FILE\0t"),
        ];
        expect.push(
            obj.read_to_commit(&null_commit_hash)
                .unwrap()
                .committed_objs_tree,
        );
        expect.push(
            obj.read_to_commit(&commit_hash1)
                .unwrap()
                .committed_objs_tree,
        );
        expect.push(
            obj.read_to_commit(&commit_hash2)
                .unwrap()
                .committed_objs_tree,
        );
        for (_, obj) in trace_obj.iter() {
            expect.push(obj.clone());
        }

        let mut expect = expect
            .into_iter()
            .collect::<HashSet<ObjHash>>()
            .into_iter()
            .collect::<Vec<ObjHash>>();
        expect.sort();
        assert_eq!(objs, expect);
    }
}
