use std::collections::{HashSet, VecDeque};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::branch::BranchName;
use crate::file_system::{FilePath, FileSystem};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::commit_hashes::CommitHashIo;
use crate::io::commit_obj::CommitObjIo;
use crate::object::commit::CommitHash;
use crate::object::tree::TreeObj;
use crate::object::ObjHash;
use crate::operation::commit::Commit;
use crate::operation::unzip::UnZip;

#[derive(Debug)]
pub struct Merge<Fs>
where
    Fs: FileSystem,
{
    head: HeadIo<Fs>,
    commit_hashes: CommitHashIo<Fs>,
    unzip: UnZip<Fs>,
    staging: StagingIo<Fs>,
    fs: Fs,
}

impl<Fs> Merge<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Merge<Fs> {
        Self {
            head: HeadIo::new(fs.clone()),
            commit_hashes: CommitHashIo::new(fs.clone()),
            unzip: UnZip::new(fs.clone()),
            staging: StagingIo::new(fs.clone()),
            fs,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MergedStatus {
    FastSource,
    FastDist,
    Merged,
    Conflicted(Vec<Conflict>),
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Conflict {
    pub file_path: FilePath,
    pub source: ObjHash,
    pub dist: ObjHash,
}

impl<Fs> Merge<Fs>
where
    Fs: FileSystem + Clone,
{
    pub fn execute(
        &self,
        source: BranchName,
        dist: BranchName,
    ) -> crate::error::Result<MergedStatus> {
        let source_head = self.read_source_head(&source)?;
        let dist_head = self.head.try_read(&dist)?;
        let source_hashes = self.commit_hashes.read_all(source_head.clone(), &None)?;
        let dist_hashes = self.commit_hashes.read_all(dist_head.clone(), &None)?;

        if source_hashes.contains(&dist_head) {
            self.head.write(&dist, &source_head)?;
            self.unzip.execute(&dist)?;
            return Ok(MergedStatus::FastSource);
        }

        if dist_hashes.contains(&source_head) {
            return Ok(MergedStatus::FastDist);
        }

        match self.inspect_merges(source.clone(), dist.clone(), source_hashes, dist_hashes)? {
            InspectStatus::CanMerge(tree) => {
                self.staging.write_tree(&tree)?;
                Commit::new(dist.clone(), self.fs.clone())
                    .execute(format!("merged {source} to {dist}"))?;
                self.unzip.execute(&dist)?;
                Ok(MergedStatus::Merged)
            }
            InspectStatus::Conflict(conflicts) => Ok(MergedStatus::Conflicted(conflicts)),
        }
    }

    fn read_source_head(&self, source: &BranchName) -> crate::error::Result<CommitHash> {
        if let Some(head) = self.head.read(source)? {
            Ok(head)
        } else {
            self.head.try_read_remote(source)
        }
    }

    fn inspect_merges(
        &self,
        source: BranchName,
        dist: BranchName,
        source_hashes: Vec<CommitHash>,
        dist_hashes: Vec<CommitHash>,
    ) -> crate::error::Result<InspectStatus> {
        let merge_origin = self.merge_origin(&source_hashes, &dist_hashes)?;
        let dist_tree = self.commit_tree(dist_hashes.into_iter().collect(), dist, &merge_origin)?;
        let mut source_tree =
            self.commit_tree(source_hashes.into_iter().collect(), source, &merge_origin)?;
        let conflicts = Vec::new();

        for (path, dist_hash) in dist_tree.iter() {
            if !source_tree.contains_key(path) {
                continue;
            }

            let source_hash = source_tree.get(path).unwrap().clone();
            if dist_hash == &source_hash {
                source_tree.remove(path);
            }
            // TODO: 現状はコンフリクトは検査せずに相手側のブランチをすべて取り込むようにします。
            // else {
            //     conflicts.push(Conflict {
            //         source: source_hash,
            //         dist: dist_hash.clone(),
            //         file_path: path.clone(),
            //     });
            // }
        }

        if conflicts.is_empty() {
            Ok(InspectStatus::CanMerge(source_tree))
        } else {
            Ok(InspectStatus::Conflict(conflicts))
        }
    }

    fn commit_tree(
        &self,
        mut commit_hashes: VecDeque<CommitHash>,
        source: BranchName,
        merge_origin: &CommitHash,
    ) -> crate::error::Result<TreeObj> {
        let commit_obj_io = CommitObjIo::new(source, self.fs.clone());
        let mut tree = TreeObj::default();

        while let Some(hash) = commit_hashes.pop_front() {
            if merge_origin == &hash {
                break;
            }
            let commit_tree = commit_obj_io.read_commit_tree(&hash)?;
            tree.replace_by(commit_tree);
        }
        Ok(tree)
    }

    fn merge_origin(
        &self,
        source_hashes: &[CommitHash],
        dist_hashes: &[CommitHash],
    ) -> crate::error::Result<CommitHash> {
        let s = source_hashes.iter().collect::<HashSet<&CommitHash>>();
        let d = dist_hashes.iter().collect::<HashSet<&CommitHash>>();
        let same_commits = s.intersection(&d).collect::<Vec<&&CommitHash>>();
        let merge_origin = source_hashes
            .iter()
            .find(|hash| same_commits.contains(&hash))
            .unwrap();
        Ok(merge_origin.clone())
    }
}

#[derive(Debug, Clone)]
enum InspectStatus {
    CanMerge(TreeObj),
    Conflict(Vec<Conflict>),
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::workspace::WorkspaceIo;
    use crate::operation::checkout::Checkout;
    use crate::operation::commit::Commit;
    use crate::operation::init::Init;
    use crate::operation::merge::{Merge, MergedStatus};
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn fast_merge() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let second = BranchName::from("second");
        Checkout::new(mock.clone()).execute(&second).unwrap();

        mock.force_write("workspace/hello.txt", b"hello");
        Stage::new(second.clone(), mock.clone())
            .execute(".")
            .unwrap();
        Commit::new(second.clone(), mock.clone())
            .execute("commit text")
            .unwrap();
        mock.delete("workspace/hello.txt").unwrap();

        Checkout::new(mock.clone())
            .execute(&BranchName::owner())
            .unwrap();
        let status = Merge::new(mock.clone())
            .execute(second, BranchName::owner())
            .unwrap();
        assert_eq!(status, MergedStatus::FastSource);
        let file = WorkspaceIo::new(mock.clone())
            .read(&FilePath::from_path("hello.txt"))
            .unwrap();
        assert!(file.is_some());
    }

    #[test]
    fn fast_merge_from_dist() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let main = BranchName::owner();
        let second = BranchName::from("second");
        Checkout::new(mock.clone()).execute(&second).unwrap();

        Checkout::new(mock.clone()).execute(&main).unwrap();

        mock.force_write("workspace/hello.txt", b"hello");
        Stage::new(main.clone(), mock.clone()).execute(".").unwrap();
        Commit::new(main.clone(), mock.clone())
            .execute("commit text")
            .unwrap();

        let status = Merge::new(mock.clone())
            .execute(second, BranchName::owner())
            .unwrap();
        assert_eq!(status, MergedStatus::FastDist);
        let file = WorkspaceIo::new(mock.clone())
            .read(&FilePath::from_path("hello.txt"))
            .unwrap();
        assert!(file.is_some());
    }

    #[test]
    fn success_merged() {
        let mock = MockFileSystem::default();

        let b1 = BranchName::owner();
        let b2 = BranchName::from("session");
        let init = Init::new(b1.clone(), mock.clone());
        let checkout = Checkout::new(mock.clone());
        init.execute().unwrap();
        checkout.execute(&b2).unwrap();
        checkout.execute(&b1).unwrap();
        mock.force_write("workspace/hello.txt", b"hello");
        Stage::new(b1.clone(), mock.clone()).execute(".").unwrap();
        Commit::new(b1.clone(), mock.clone())
            .execute("TEXT")
            .unwrap();

        checkout.execute(&b2).unwrap();
        mock.force_write("workspace/test.txt", b"HELLO");
        Stage::new(b2.clone(), mock.clone()).execute(".").unwrap();
        Commit::new(b2.clone(), mock.clone())
            .execute("TEXT")
            .unwrap();

        let merge = Merge::new(mock.clone());
        let status = merge.execute(b1, b2).unwrap();
        assert!(matches!(status, MergedStatus::Merged));

        assert!(mock.read_file("workspace/hello.txt").unwrap().is_some());
        assert!(mock.read_file("workspace/test.txt").unwrap().is_some());
    }

    // TODO: 現状はコンフリクト関連が未実装のため実装された際にこのテストも展開します。
    //     #[test]
    // fn conflicts() {
    //     let mock = MockFileSystem::default();
    //
    //     let b1 = BranchName::owner();
    //     let b2 = BranchName::from("session");
    //     let init = Init::new(b1.clone(), mock.clone());
    //     let checkout = Checkout::new(mock.clone());
    //     init.execute().unwrap();
    //     checkout.execute(&b2).unwrap();
    //     checkout.execute(&b1).unwrap();
    //     mock.force_write("workspace/hello.txt", b"hello");
    //     Stage::new(b1.clone(), mock.clone()).execute(".").unwrap();
    //     Commit::new(b1.clone(), mock.clone()).execute("TEXT").unwrap();
    //
    //     checkout.execute(&b2).unwrap();
    //     mock.force_write("workspace/hello.txt", b"HELLO");
    //     Stage::new(b2.clone(), mock.clone()).execute(".").unwrap();
    //     Commit::new(b2.clone(), mock.clone()).execute("TEXT").unwrap();
    //
    //     let merge = Merge::new(mock.clone());
    //     let status = merge.execute(b1, b2).unwrap();
    //     assert!(matches!(status, MergedStatus::Conflicted(_)));
    // }
}
