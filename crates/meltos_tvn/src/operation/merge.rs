use crate::branch::BranchName;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::commit_obj::CommitObjIo;
use crate::io::workspace::WorkspaceIo;
use crate::operation::unzip::UnZip;

pub struct Merge<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    fs: FsIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
    commits_obj: CommitObjIo<Fs, Io>,
    unzip: UnZip<Fs, Io>,
}

impl<Fs, Io> Merge<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(fs: Fs) -> Merge<Fs, Io> {
        Self {
            head: HeadIo::new(fs.clone()),
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
        Io: std::io::Write + std::io::Read
{
    pub fn execute(
        &self,
        source: BranchName,
        dist: BranchName,
    ) -> crate::error::Result<MergedStatus> {
        let source_head = self.head.try_read(&source)?;
        let dist_head = self.head.try_read(&dist)?;
        let source_hashes = self.commits_obj.read_hashes(source_head.clone(), &None)?;
        let dist_hashes = self.commits_obj.read_hashes(dist_head.clone(), &None)?;
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
    use crate::operation::merge::Merge;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn fast_merge() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());

        let second = BranchName::from("second");
        Checkout::new(mock.clone()).execute(&second).unwrap();

        mock.force_write("./hello.txt", b"hello");
        Stage::new(second.clone(), mock.clone()).execute(".").unwrap();
        Commit::new(second.clone(), mock.clone()).execute("commit text").unwrap();
        mock.delete("./hello.txt").unwrap();

        Checkout::new(mock.clone()).execute(&BranchName::main()).unwrap();
        let status = Merge::new(mock.clone()).execute(second, BranchName::main()).unwrap();
        println!("{mock:?}");
        println!("status={status:?}");
        let file = WorkspaceIo::new(mock.clone()).read(&FilePath::from_path("./hello.txt")).unwrap();
        assert!(file.is_some());
    }
}