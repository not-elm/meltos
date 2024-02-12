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
    commit: Commit<Fs>,
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
            commit: Commit::new(fs.clone()),
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
    #[inline]
    pub async fn execute_from_branch(
        &self,
        source: BranchName,
        dist: BranchName,
    ) -> crate::error::Result<MergedStatus> {
        let source_head = self.read_source_head(&source).await?;
        self.execute(source_head, dist).await
    }

    pub async fn execute(
        &self,
        source: CommitHash,
        dist: BranchName,
    ) -> crate::error::Result<MergedStatus> {
        let dist_head = self.head.try_read(&dist).await?;
        let source_hashes = self.commit_hashes.read_all(source.clone(), &None).await?;
        let dist_hashes = self
            .commit_hashes
            .read_all(dist_head.clone(), &None)
            .await?;

        if source_hashes.contains(&dist_head) {
            self.head.write(&dist, &source).await?;
            self.unzip.execute(&dist).await?;
            return Ok(MergedStatus::FastSource);
        }

        if dist_hashes.contains(&source) {
            return Ok(MergedStatus::FastDist);
        }

        match self.inspect_merges(source_hashes, dist_hashes).await? {
            InspectStatus::CanMerge(tree) => {
                self.head.write(&dist, &source).await?;
                self.staging.write_tree(&tree).await?;
                self.commit
                    .execute(&dist, format!("merged {source} to {dist}"))
                    .await?;
                self.unzip.execute(&dist).await?;
                Ok(MergedStatus::Merged)
            }
            InspectStatus::Conflict(conflicts) => Ok(MergedStatus::Conflicted(conflicts)),
        }
    }

    async fn read_source_head(&self, source: &BranchName) -> crate::error::Result<CommitHash> {
        if let Some(head) = self.head.read(source).await? {
            Ok(head)
        } else {
            self.head.try_read_remote(source).await
        }
    }

    async fn inspect_merges(
        &self,
        source_hashes: Vec<CommitHash>,
        dist_hashes: Vec<CommitHash>,
    ) -> crate::error::Result<InspectStatus> {
        let merge_origin = self.merge_origin(&source_hashes, &dist_hashes)?;
        let mut dist_tree = self
            .commit_tree(dist_hashes.into_iter().collect(), &merge_origin)
            .await?;
        let source_tree = self
            .commit_tree(source_hashes.into_iter().collect(), &merge_origin)
            .await?;
        let conflicts = Vec::new();

        for (path, _) in source_tree.iter() {
            if dist_tree.contains_key(path) {
                dist_tree.remove(path);
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
            Ok(InspectStatus::CanMerge(dist_tree))
        } else {
            Ok(InspectStatus::Conflict(conflicts))
        }
    }

    async fn commit_tree(
        &self,
        mut commit_hashes: VecDeque<CommitHash>,
        merge_origin: &CommitHash,
    ) -> crate::error::Result<TreeObj> {
        let commit_obj_io = CommitObjIo::new(self.fs.clone());
        let mut tree = TreeObj::default();

        while let Some(hash) = commit_hashes.pop_front() {
            if merge_origin == &hash {
                break;
            }
            let commit_tree = commit_obj_io.read_commit_tree(&hash).await?;
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
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::{FilePath, FileSystem};
    use crate::io::workspace::WorkspaceIo;
    use crate::operation::checkout::Checkout;
    use crate::operation::commit::Commit;
    use crate::operation::init::Init;
    use crate::operation::merge::{Merge, MergedStatus};
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn fast_merge() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;

        let second = BranchName::from("second");
        Checkout::new(fs.clone()).execute(&second).await.unwrap();

        fs.write_sync("hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&second, ".").await.unwrap();
        Commit::new(fs.clone())
            .execute(&second, "commit text")
            .await
            .unwrap();
        fs.delete("hello.txt").await.unwrap();

        Checkout::new(fs.clone())
            .execute(&BranchName::owner())
            .await
            .unwrap();
        let status = Merge::new(fs.clone())
            .execute_from_branch(second, BranchName::owner())
            .await
            .unwrap();
        assert_eq!(status, MergedStatus::FastSource);
        let file = WorkspaceIo::new(fs.clone())
            .read(&FilePath::from_path("hello.txt"))
            .await
            .unwrap();
        assert!(file.is_some());
    }

    #[tokio::test]
    async fn fast_merge_from_dist() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;

        let branch = BranchName::owner();
        let second = BranchName::from("second");
        Checkout::new(fs.clone()).execute(&second).await.unwrap();

        Checkout::new(fs.clone()).execute(&branch).await.unwrap();

        fs.write_sync("hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone())
            .execute(&branch, "commit text")
            .await
            .unwrap();

        let status = Merge::new(fs.clone())
            .execute_from_branch(second, BranchName::owner())
            .await
            .unwrap();
        assert_eq!(status, MergedStatus::FastDist);
        let file = WorkspaceIo::new(fs.clone())
            .read(&FilePath::from_path("hello.txt"))
            .await
            .unwrap();
        assert!(file.is_some());
    }

    #[tokio::test]
    async fn success_merged() {
        let fs = MemoryFileSystem::default();

        let b1 = BranchName::owner();
        let b2 = BranchName::from("session");
        let init = Init::new(fs.clone());
        let checkout = Checkout::new(fs.clone());
        init.execute(&b1).await.unwrap();
        checkout.execute(&b2).await.unwrap();
        checkout.execute(&b1).await.unwrap();
        fs.write_sync("hello.txt", b"hello");
        Stage::new(fs.clone()).execute(&b2, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&b2, "TEXT").await.unwrap();

        checkout.execute(&b2).await.unwrap();
        fs.write_sync("test.txt", b"HELLO");
        Stage::new(fs.clone()).execute(&b1, ".").await.unwrap();
        Commit::new(fs.clone()).execute(&b1, "TEXT").await.unwrap();

        let merge = Merge::new(fs.clone());
        let status = merge.execute_from_branch(b1, b2).await.unwrap();
        assert!(matches!(status, MergedStatus::Merged));

        assert!(fs.read_file("hello.txt").await.unwrap().is_some());
        assert!(fs.read_file("test.txt").await.unwrap().is_some());
    }

    // TODO: 現状はコンフリクト関連が未実装のため実装された際にこのテストも展開します。
    //     #[test]
    // fn conflicts() {
    //     let fs = MockFileSystem::default();
    //
    //     let b1 = BranchName::owner();
    //     let b2 = BranchName::from("session");
    //     let init = Init::new(b1.clone(), fs.clone());
    //     let checkout = Checkout::new(fs.clone());
    //     init.execute().unwrap();
    //     checkout.execute(&b2).unwrap();
    //     checkout.execute(&b1).unwrap();
    //     fs.force_write("hello.txt", b"hello");
    //     Stage::new(b1.clone(), fs.clone()).execute(".").unwrap();
    //     Commit::new(b1.clone(), fs.clone()).execute("TEXT").unwrap();
    //
    //     checkout.execute(&b2).unwrap();
    //     fs.force_write("hello.txt", b"HELLO");
    //     Stage::new(b2.clone(), fs.clone()).execute(".").unwrap();
    //     Commit::new(b2.clone(), fs.clone()).execute("TEXT").unwrap();
    //
    //     let merge = Merge::new(fs.clone());
    //     let status = merge.execute(b1, b2).unwrap();
    //     assert!(matches!(status, MergedStatus::Conflicted(_)));
    // }
}
