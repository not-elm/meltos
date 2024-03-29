use std::path::Path;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::commit::CommitHash;
use crate::object::{CompressedBuf, ObjHash};

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Bundle {
    pub traces: Vec<BundleTrace>,
    pub objs: Vec<BundleObject>,
    pub branches: Vec<BundleBranch>,
}

#[wasm_bindgen]
impl Bundle {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(
        traces: Vec<BundleTrace>,
        objs: Vec<BundleObject>,
        branches: Vec<BundleBranch>,
    ) -> Self {
        Self {
            traces,
            objs,
            branches,
        }
    }
}

impl Bundle {
    #[inline(always)]
    pub fn obj_data_size(&self) -> usize {
        self.objs.iter().map(|obj| obj.compressed_buf.0.len()).sum()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BundleTrace {
    pub commit_hash: CommitHash,
    pub obj_hash: ObjHash,
}

#[wasm_bindgen]
impl BundleTrace {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(commit_hash: String, obj_hash: String) -> Self {
        Self {
            commit_hash: CommitHash(ObjHash(commit_hash)),
            obj_hash: ObjHash(obj_hash),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BundleObject {
    pub hash: ObjHash,
    pub compressed_buf: CompressedBuf,
}

#[wasm_bindgen]
impl BundleObject {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(hash: String, compressed_buf: Vec<u8>) -> Self {
        Self {
            hash: ObjHash(hash),
            compressed_buf: CompressedBuf(compressed_buf),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BundleBranch {
    pub branch_name: BranchName,
    pub commits: Vec<CommitHash>,
}

#[wasm_bindgen]
impl BundleBranch {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(branch_name: String, commits: Vec<String>) -> Self {
        Self {
            branch_name: BranchName(branch_name),
            commits: commits
                .into_iter()
                .map(|hash| CommitHash(ObjHash(hash)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BundleIo<Fs>
where
    Fs: FileSystem,
{
    object: ObjIo<Fs>,
    trace: TraceIo<Fs>,
    fs: Fs,
}

impl<Fs> BundleIo<Fs>
where
    Fs: FileSystem + Clone,
{
    #[inline]
    pub fn new(fs: Fs) -> BundleIo<Fs> {
        Self {
            object: ObjIo::new(fs.clone()),
            trace: TraceIo::new(fs.clone()),
            fs,
        }
    }

    pub async fn create(&self) -> error::Result<Bundle> {
        let branches = self.read_branch_heads().await?;
        Ok(Bundle {
            branches,
            objs: self.object.read_all().await?,
            traces: self.trace.read_all().await?,
        })
    }

    async fn read_branch_heads(&self) -> error::Result<Vec<BundleBranch>> {
        let head_files = self.read_all_branch_head_path().await?;
        let mut branches = Vec::with_capacity(head_files.len());
        for path in head_files {
            let Some(branch_name) = Path::new(&path).file_name().and_then(|name| name.to_str())
            else {
                continue;
            };

            let branch_name = BranchName::from(branch_name);
            let head = HeadIo::new(self.fs.clone()).try_read(&branch_name).await?;
            branches.push(BundleBranch {
                commits: vec![head],
                branch_name,
            });
        }

        Ok(branches)
    }

    #[inline]
    async fn read_all_branch_head_path(&self) -> error::Result<Vec<String>> {
        Ok(self.fs.all_files_in(".meltos/refs/heads").await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FileSystem;
    use crate::io::atomic::work_branch::WorkingIo;
    use crate::io::bundle::BundleIo;
    use crate::operation::commit::Commit;
    use crate::operation::new_branch::NewBranch;
    use crate::operation::stage::Stage;
    use crate::tests::init_owner_branch;

    #[tokio::test]
    async fn read_head() {
        let fs = MemoryFileSystem::default();
        let bundle_io = BundleIo::new(fs.clone());
        let null_commit_hash = init_owner_branch(fs.clone()).await;
        let bundle = bundle_io.create().await.unwrap();
        assert_eq!(bundle.branches.len(), 1);
        assert_eq!(&bundle.branches[0].branch_name, &BranchName::owner());
        assert_eq!(&bundle.branches[0].commits[0], &null_commit_hash);
    }

    #[tokio::test]
    async fn read_2_heads() {
        let fs = MemoryFileSystem::default();
        let new_branch = NewBranch::new(fs.clone());
        let bundle_io = BundleIo::new(fs.clone());

        let null_commit = init_owner_branch(fs.clone()).await;
        new_branch
            .execute(BranchName::owner(), BranchName::from("branch2"))
            .await
            .unwrap();

        let mut bundle = bundle_io.create().await.unwrap();
        assert_eq!(bundle.branches.len(), 2);
        bundle.branches.sort();
        assert_eq!(
            &bundle.branches[0].branch_name,
            &BranchName::from("branch2")
        );
        assert_eq!(&bundle.branches[1].branch_name, &BranchName::owner());

        assert_eq!(&bundle.branches[0].commits[0], &null_commit);
        assert_eq!(&bundle.branches[1].commits[0], &null_commit);

        let working = WorkingIo::new(fs.clone());
        let stage = Stage::new(fs.clone());
        let commit = Commit::new(fs.clone());
        let branch = BranchName::from("branch2");

        working.write(&BranchName::from("branch2")).await.unwrap();
        fs.write_file("hello.txt", b"hello")
            .await
            .unwrap();
        stage.execute(&branch, ".").await.unwrap();
        let commit_hash = commit.execute(&branch, "text").await.unwrap();
        let mut bundle = bundle_io.create().await.unwrap();
        bundle.branches.sort();
        assert_eq!(
            &bundle.branches[0].branch_name,
            &BranchName::from("branch2")
        );
        assert_eq!(&bundle.branches[1].branch_name, &BranchName::owner());

        assert_eq!(&bundle.branches[0].commits[0], &commit_hash);
        assert_eq!(&bundle.branches[1].commits[0], &null_commit);
    }

    #[tokio::test]
    async fn read_all_objs() {
        let fs = MemoryFileSystem::default();
        init_owner_branch(fs.clone()).await;
        let branch = BranchName::owner();
        fs.write_file("hello.txt", b"hello")
            .await
            .unwrap();
        Stage::new(fs.clone()).execute(&branch, ".").await.unwrap();
        Commit::new(fs.clone())
            .execute(&branch, "commit")
            .await
            .unwrap();
        let bundle = BundleIo::new(fs.clone()).create().await.unwrap();
        let objs_count = fs.all_files_in(".meltos/objects").await.unwrap().len();

        assert_eq!(objs_count, bundle.objs.len());
    }
}
