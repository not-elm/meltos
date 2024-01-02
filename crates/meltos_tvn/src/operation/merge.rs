use wasm_bindgen::prelude::wasm_bindgen;

use crate::branch::BranchName;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::commit_hashes::CommitHashIo;
use crate::object::commit::CommitHash;
use crate::operation::unzip::UnZip;

#[derive(Debug)]
pub struct Merge<Fs>
    where
        Fs: FileSystem,
{
    head: HeadIo<Fs>,
    commit_hashes: CommitHashIo<Fs>,
    unzip: UnZip<Fs>,
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
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum MergedStatus {
    FastSource,
    FastDist,
}

impl<Fs> Merge<Fs>
    where
        Fs: FileSystem,
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

        todo!();
    }

    fn read_source_head(&self, source: &BranchName) -> crate::error::Result<CommitHash> {
        if let Some(head) = self.head.read(source)? {
            Ok(head)
        } else {
            self.head.try_read_remote(source)
        }
    }

    // fn commit_objs(
    //     &self,
    //     source_hashes: Vec<CommitHash>,
    //     dist_hashes: Vec<CommitHash>,
    // ) -> crate::error::Result {
    //     let merge_origin = self.merge_origin(&source_hashes, &dist_hashes)?;
    //     let dist = self.commit_tree(dist_hashes, &merge_origin)?;
    //     let mut source = self.commit_tree(source_hashes, &merge_origin)?;
    //
    //     for (path, dist_hash) in dist.iter() {
    //         if !source.contains_key(path) {
    //             continue;
    //         }
    //
    //         let source_hash = source.get(path).unwrap().clone();
    //         if dist_hash == &source_hash {
    //             source.remove(path);
    //         }
    //     }
    //     todo!();
    // }
    //
    // fn commit_tree(
    //     &self,
    //     mut commit_hashes: Vec<CommitHash>,
    //     merge_origin: &CommitHash,
    // ) -> crate::error::Result<TreeObj> {
    //     let mut tree = TreeObj::default();
    //     while let Some(hash) = commit_hashes.pop() {
    //         if merge_origin == &hash {
    //             break;
    //         }
    //         let commit_tree = self.commits_obj.read_commit_tree(&hash)?;
    //         tree.replace_by(commit_tree);
    //     }
    //     Ok(tree)
    // }
    //
    // fn merge_origin(
    //     &self,
    //     source_hashes: &[CommitHash],
    //     dist_hashes: &[CommitHash],
    // ) -> crate::error::Result<CommitHash> {
    //     let s = source_hashes.iter().collect::<HashSet<&CommitHash>>();
    //     let d = dist_hashes.iter().collect::<HashSet<&CommitHash>>();
    //     let same_commits = s.intersection(&d).collect::<Vec<&&CommitHash>>();
    //     let merge_origin = source_hashes
    //         .iter()
    //         .find(|hash| same_commits.contains(&hash))
    //         .unwrap();
    //     Ok(merge_origin.clone())
    // }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MergeConfig {}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::workspace::WorkspaceIo;
    use crate::operation::checkout::Checkout;
    use crate::operation::commit::Commit;
    use crate::operation::merge::{Merge, MergedStatus};
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn fast_merge() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let second = BranchName::from("second");
        Checkout::new(mock.clone()).execute(&second).unwrap();

        mock.force_write("./workspace/hello.txt", b"hello");
        Stage::new(second.clone(), mock.clone())
            .execute(".")
            .unwrap();
        Commit::new(second.clone(), mock.clone())
            .execute("commit text")
            .unwrap();
        mock.delete("./workspace/hello.txt").unwrap();

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

        mock.force_write("./workspace/hello.txt", b"hello");
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
}
