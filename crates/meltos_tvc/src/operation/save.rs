use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::bundle::{Bundle, BundleBranch, BundleObject, BundleTrace};
use crate::object::commit::CommitHash;

#[derive(Debug, Clone)]
pub struct Save<Fs>
    where
        Fs: FileSystem,
{
    trace: TraceIo<Fs>,
    object: ObjIo<Fs>,
    head: HeadIo<Fs>,
}

impl<Fs> Save<Fs>
    where
        Fs: FileSystem + Clone,
{
    pub fn new(fs: Fs) -> Save<Fs> {
        Self {
            trace: TraceIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs),
        }
    }

    /// * write objs.
    /// * write head.
    /// * write traces related to commits.
    pub async fn execute(&self, bundle: Bundle) -> error::Result {
        self.write_objs(bundle.objs).await?;
        self.write_branches(&bundle.branches).await?;
        self.write_traces(bundle.traces).await
    }

    async fn write_objs(&self, objs: Vec<BundleObject>) -> error::Result {
        for obj in objs {
            self.object.write(&obj.hash, &obj.compressed_buf).await?;
        }

        Ok(())
    }

    async fn write_branches(&self, branches: &[BundleBranch]) -> error::Result {
        for branch in branches {
            self
                .write_head(
                    &branch.branch_name,
                    &branch.commits[branch.commits.len() - 1],
                )
                .await?;
        }
        Ok(())
    }

    #[inline]
    async fn write_head(&self, branch: &BranchName, head_hash: &CommitHash) -> error::Result {
        self.head.write(branch, head_hash).await?;
        Ok(())
    }

    async fn write_traces(&self, traces: Vec<BundleTrace>) -> error::Result {
        for trace in traces {
            self.trace.write(&trace.commit_hash, &trace.obj_hash).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::encode::Encodable;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::bundle::{Bundle, BundleBranch};
    use crate::object::commit::CommitHash;
    use crate::object::ObjHash;
    use crate::operation::save::Save;

    #[tokio::test]
    async fn created_head_file() {
        let fs = MockFileSystem::default();
        let save = Save::new(fs.clone());

        let head = CommitHash(ObjHash::new(b"commit hash"));
        let bundle = Bundle {
            branches: vec![BundleBranch {
                branch_name: BranchName::owner(),
                commits: vec![head.clone()],
            }],
            traces: Vec::with_capacity(0),
            objs: Vec::with_capacity(0),
        };
        save.execute(bundle).await.unwrap();
        let actual = fs.try_read_file(".meltos/refs/heads/owner").await.unwrap();
        assert_eq!(actual, head.encode().unwrap());
    }
}
