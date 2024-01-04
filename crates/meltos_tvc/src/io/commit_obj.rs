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
use crate::object::tree::TreeObj;
use crate::object::ObjHash;

#[derive(Debug, Clone)]
pub struct CommitObjIo<Fs>
where
    Fs: FileSystem,
{
    head: HeadIo<Fs>,
    object: ObjIo<Fs>,
    local_commits: LocalCommitsIo<Fs>,
    trace_tree: TraceTreeIo<Fs>,
    branch_name: BranchName,
}

impl<Fs> CommitObjIo<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(branch_name: BranchName, fs: Fs) -> CommitObjIo<Fs> {
        CommitObjIo {
            head: HeadIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            local_commits: LocalCommitsIo::new(branch_name.clone(), fs.clone()),
            trace_tree: TraceTreeIo::new(fs),
            branch_name,
        }
    }
}

impl<Fs> CommitObjIo<Fs>
where
    Fs: FileSystem,
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

    pub fn read_commit_tree(&self, commit_hash: &ObjHash) -> error::Result<TreeObj> {
        let commit = self.read(commit_hash)?;
        let tree = self.object.read_to_tree(&commit.committed_objs_tree)?;
        Ok(tree)
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

    pub fn read_objs_associated_with_local_commits(&self) -> error::Result<Vec<BundleObject>> {
        let local_commits = self.local_commits.try_read()?;
        let head = self.head.read(&self.branch_name)?;
        let from = local_commits.0[local_commits.0.len() - 1].clone();
        let obj_hashes = self.read_obj_hashes(from, &head)?;
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

    pub fn read_obj_hashes(
        &self,
        from: CommitHash,
        to: &Option<CommitHash>,
    ) -> error::Result<HashSet<ObjHash>> {
        let mut obj_hashes = HashSet::new();
        self._read_hashes(&mut obj_hashes, from, to)?;
        Ok(obj_hashes)
    }

    fn _read_hashes(
        &self,
        obj_hashes: &mut HashSet<ObjHash>,
        from: CommitHash,
        to: &Option<CommitHash>,
    ) -> error::Result {
        let tree = self.trace_tree.read(&from)?;
        self.scan_commit_obj(obj_hashes, from, to)?;
        for obj_hash in tree.0.into_values() {
            obj_hashes.insert(obj_hash);
        }
        Ok(())
    }

    fn scan_commit_obj(
        &self,
        obj_hashes: &mut HashSet<ObjHash>,
        commit_hash: CommitHash,
        to: &Option<CommitHash>,
    ) -> error::Result {
        let commit_obj = self.read(&commit_hash)?;
        self.scan_commit_tree(obj_hashes, &commit_obj, to)?;
        obj_hashes.insert(commit_hash.0);

        Ok(())
    }

    fn scan_commit_tree(
        &self,
        obj_hashes: &mut HashSet<ObjHash>,
        commit_obj: &CommitObj,
        to: &Option<CommitHash>,
    ) -> error::Result {
        let tree = self.object.read_to_tree(&commit_obj.committed_objs_tree)?;
        obj_hashes.insert(commit_obj.committed_objs_tree.clone());
        for hash in tree.0.into_values() {
            obj_hashes.insert(hash);
        }
        if !to.as_ref().is_some_and(|p| commit_obj.parents.contains(p)) {
            for hash in commit_obj.parents.iter() {
                self._read_hashes(obj_hashes, hash.clone(), to)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::commit_obj::CommitObjIo;
    use crate::io::trace_tree::TraceTreeIo;
    use crate::object::ObjHash;
    use crate::operation::commit::Commit;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn local_commits_is_empty_if_not_committed() {
        let local_commit_objs = CommitObjIo::new(BranchName::owner(), MockFileSystem::default())
            .read_local_commits()
            .unwrap();
        assert_eq!(local_commit_objs, vec![]);
    }

    #[test]
    fn local_commit_count_is_2() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        let branch = BranchName::owner();
        let stage = Stage::new(branch.clone(), mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let commit_obj = CommitObjIo::new(branch, mock.clone());

        mock.write_file("./workspace/hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        mock.write_file("./workspace/hello.txt", b"hello2").unwrap();
        stage.execute(".").unwrap();
        commit.execute("commit text").unwrap();

        let local_commits = commit_obj.read_local_commits().unwrap();
        assert_eq!(local_commits.len(), 3);
    }

    #[test]
    fn read_objs_associated_with_all_commits() {
        let mock = MockFileSystem::default();
        let null_commit_hash = init_main_branch(mock.clone());
        let branch = BranchName::owner();
        let stage = Stage::new(branch.clone(), mock.clone());
        let trace = TraceTreeIo::new(mock.clone());
        let commit = Commit::new(branch.clone(), mock.clone());
        let obj = ObjIo::new(mock.clone());
        let commit_obj = CommitObjIo::new(branch, mock.clone());

        mock.write_file("./workspace/hello/hello", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash1 = commit.execute("commit text").unwrap();

        mock.write_file("./workspace/src/sample", b"sample").unwrap();
        mock.write_file("./workspace/t", b"t").unwrap();
        stage.execute(".").unwrap();
        let commit_hash2 = commit.execute("commit text").unwrap();

        let mut objs = commit_obj
            .read_obj_hashes(commit_hash2.clone(), &None)
            .unwrap()
            .into_iter()
            .collect::<Vec<ObjHash>>();

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
