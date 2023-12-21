use std::collections::HashSet;

use crate::branch::BranchName;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::commit_hashes::CommitHashIo;
use crate::io::commit_obj::CommitObjIo;
use crate::object::commit::CommitHash;
use crate::object::tree::TreeObj;
use crate::operation::unzip::UnZip;

pub struct Merge<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    fs: FsIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
    commit_hashes: CommitHashIo<Fs, Io>,
    commits_obj: CommitObjIo<Fs, Io>,
    unzip: UnZip<Fs, Io>,
}

impl<Fs, Io> Merge<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> Merge<Fs, Io> {
        Self {
            head: HeadIo::new(fs.clone()),
            commit_hashes: CommitHashIo::new(fs.clone()),
            commits_obj: CommitObjIo::new(BranchName::main(), fs.clone()),
            unzip: UnZip::new(fs.clone()),
            fs: FsIo::new(fs),
        }
    }
}


#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum MergedStatus {
    FastSource,
    FastDist,
}


impl<Fs, Io> Merge<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn execute(
        &self,
        source: BranchName,
        dist: BranchName,
    ) -> crate::error::Result<MergedStatus> {
        let source_head = self.head.try_read(&source)?;
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


    fn commit_objs(
        &self,
        source_hashes: Vec<CommitHash>,
        dist_hashes: Vec<CommitHash>,
    ) -> crate::error::Result {
        let merge_origin = self.merge_origin(&source_hashes, &dist_hashes)?;
        let dist = self.commit_tree(dist_hashes, &merge_origin)?;
        let mut source = self.commit_tree(source_hashes, &merge_origin)?;

        for (path, dist_hash) in dist.iter() {
            if !source.contains_key(path) {
                continue;
            }

            let source_hash = source.get(path).unwrap().clone();
            if dist_hash == &source_hash {
                source.remove(path);
            } else {}
        }
        todo!();
    }


    fn commit_tree(
        &self,
        mut commit_hashes: Vec<CommitHash>,
        merge_origin: &CommitHash,
    ) -> crate::error::Result<TreeObj> {
        let mut tree = TreeObj::default();
        while let Some(hash) = commit_hashes.pop() {
            if merge_origin == &hash {
                break;
            }
            let commit_tree = self.commits_obj.read_commit_tree(&hash)?;
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

        mock.force_write("./hello.txt", b"hello");
        Stage::new(second.clone(), mock.clone())
            .execute(".")
            .unwrap();
        Commit::new(second.clone(), mock.clone())
            .execute("commit text")
            .unwrap();
        mock.delete("./hello.txt").unwrap();

        Checkout::new(mock.clone())
            .execute(&BranchName::main())
            .unwrap();
        let status = Merge::new(mock.clone())
            .execute(second, BranchName::main())
            .unwrap();
        assert_eq!(status, MergedStatus::FastSource);
        let file = WorkspaceIo::new(mock.clone())
            .read(&FilePath::from_path("./hello.txt"))
            .unwrap();
        assert!(file.is_some());
    }


    #[test]
    fn fast_merge_from_dist() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let main = BranchName::main();
        let second = BranchName::from("second");
        Checkout::new(mock.clone()).execute(&second).unwrap();

        Checkout::new(mock.clone()).execute(&main).unwrap();

        mock.force_write("./hello.txt", b"hello");
        Stage::new(main.clone(), mock.clone()).execute(".").unwrap();
        Commit::new(main.clone(), mock.clone())
            .execute("commit text")
            .unwrap();

        let status = Merge::new(mock.clone())
            .execute(second, BranchName::main())
            .unwrap();
        assert_eq!(status, MergedStatus::FastDist);
        let file = WorkspaceIo::new(mock.clone())
            .read(&FilePath::from_path("./hello.txt"))
            .unwrap();
        assert!(file.is_some());
    }
}
