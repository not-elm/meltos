use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::branch::BranchName;
use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::{CompressedBuf, ObjHash};
use crate::object::commit::CommitHash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub traces: Vec<(CommitHash, ObjHash)>,
    pub objs: Vec<CompressedBuf>,
    pub branches: Vec<BranchHead>,
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BranchHead {
    pub branch_name: BranchName,
    pub head: CommitHash,
}


#[derive(Debug)]
pub struct BundleIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    object: ObjIo<Fs, Io>,
    trace: TraceIo<Fs, Io>,
    fs: FsIo<Fs, Io>,
}


impl<Fs, Io> BundleIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new(fs: Fs) -> BundleIo<Fs, Io> {
        Self {
            object: ObjIo::new(fs.clone()),
            trace: TraceIo::new(fs.clone()),
            fs: FsIo::new(fs),
        }
    }


    pub fn create(&self) -> error::Result<Bundle> {
        let branches = self.read_branch_heads()?;
        Ok(Bundle {
            branches,
            objs: self.object.read_all()?,
            traces: self.trace.read_all()?,
        })
    }


    fn read_branch_heads(&self) -> error::Result<Vec<BranchHead>> {
        let head_files = self.read_all_branch_head_path()?;
        let mut branches = Vec::with_capacity(head_files.len());
        for path in head_files {
            let Some(branch_name) = Path::new(&path)
                .parent()
                .and_then(|dir| dir.file_name())
                .and_then(|name| name.to_str())
                else {
                    continue;
                };

            let branch_name = BranchName::from(branch_name);
            let head = HeadIo::new(branch_name.clone(), self.fs.clone()).read()?;
            branches.push(BranchHead {
                head,
                branch_name,
            });
        }

        Ok(branches)
    }


    fn read_all_branch_head_path(&self) -> error::Result<Vec<String>> {
        Ok(self
            .fs
            .all_file_path(".meltos/branches")?
            .into_iter()
            .filter(|path| {
                let Some(file_name) = Path::new(path).file_name().and_then(|path| path.to_str())
                    else {
                        return false;
                    };
                file_name == "HEAD"
            })
            .collect())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::work_branch::WorkingIo;
    use crate::io::bundle::BundleIo;
    use crate::operation::commit::Commit;
    use crate::operation::new_branch::NewBranch;
    use crate::operation::stage::Stage;
    use crate::tests::init_main_branch;

    #[test]
    fn read_head() {
        let mock = MockFileSystem::default();
        let bundle_io = BundleIo::new(mock.clone());
        let null_commit_hash = init_main_branch(mock.clone());
        let bundle = bundle_io.create().unwrap();
        assert_eq!(bundle.branches.len(), 1);
        assert_eq!(&bundle.branches[0].branch_name, &BranchName::main());
        assert_eq!(&bundle.branches[0].head, &null_commit_hash);
    }

    #[test]
    fn read_2_heads() {
        let mock = MockFileSystem::default();
        let new_branch = NewBranch::new(mock.clone());
        let bundle_io = BundleIo::new(mock.clone());

        let null_commit = init_main_branch(mock.clone());
        new_branch
            .execute(BranchName::main(), BranchName::from("branch2"))
            .unwrap();

        let mut bundle = bundle_io.create().unwrap();
        assert_eq!(bundle.branches.len(), 2);
        bundle.branches.sort();
        assert_eq!(
            &bundle.branches[0].branch_name,
            &BranchName::from("branch2")
        );
        assert_eq!(&bundle.branches[1].branch_name, &BranchName::main());

        assert_eq!(&bundle.branches[0].head, &null_commit);
        assert_eq!(&bundle.branches[1].head, &null_commit);

        let working = WorkingIo::new(mock.clone());
        let stage = Stage::new(BranchName::from("branch2"), mock.clone());
        let commit = Commit::new(BranchName::from("branch2"), mock.clone());
        working.write(&BranchName::from("branch2")).unwrap();
        mock.write("./hello.txt", b"hello").unwrap();
        stage.execute(".").unwrap();
        let commit_hash = commit.execute("text").unwrap();
        let mut bundle = bundle_io.create().unwrap();
        bundle.branches.sort();
        assert_eq!(
            &bundle.branches[0].branch_name,
            &BranchName::from("branch2")
        );
        assert_eq!(&bundle.branches[1].branch_name, &BranchName::main());

        assert_eq!(&bundle.branches[0].head, &commit_hash);
        assert_eq!(&bundle.branches[1].head, &null_commit);
    }


    #[test]
    fn read_all_objs() {
        let mock = MockFileSystem::default();
        init_main_branch(mock.clone());
        mock.write("./hello.txt", b"hello").unwrap();

        Stage::new(BranchName::main(), mock.clone()).execute(".").unwrap();
        Commit::new(BranchName::main(), mock.clone()).execute("commit").unwrap();
        let bundle = BundleIo::new(mock.clone()).create().unwrap();
        let objs_count = mock
            .all_file_path("./.meltos/objects/").unwrap()
            .len();

        assert_eq!(objs_count, bundle.objs.len());
    }
}