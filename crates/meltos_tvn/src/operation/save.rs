use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::commit::CommitHash;
use crate::object::ObjHash;
use crate::operation::push::PushParam;

#[derive(Debug, Clone)]
pub struct Save<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    trace: TraceIo<Fs, Io>,
    fs: Fs,
}


impl<Fs, Io> Save<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> Save<Fs, Io> {
        Self {
            trace: TraceIo::new(fs.clone()),
            fs,
        }
    }


    /// * write head.
    /// * write traces related to commits.
    pub fn execute(&self, push_param: PushParam) -> error::Result {
        self.write_head(push_param.branch, push_param.head)?;
        self.write_traces(push_param.traces)
    }


    fn write_head(&self, branch: BranchName, head_hash: CommitHash) -> error::Result {
        let head = HeadIo::new(branch, self.fs.clone());
        head.write(head_hash)?;
        Ok(())
    }

    fn write_traces(&self, traces: Vec<(CommitHash, ObjHash)>) -> error::Result {
        for (commit_hash, trace_hash) in traces {
            self.trace.write(&commit_hash, &trace_hash)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::encode::Encodable;
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use crate::object::commit::CommitHash;
    use crate::object::ObjHash;
    use crate::operation::push::PushParam;
    use crate::operation::save::Save;

    #[test]
    fn created_head_file() {
        let mock = MockFileSystem::default();
        let save = Save::new(mock.clone());

        let head = CommitHash(ObjHash::new(b"commit hash"));
        let push_param = PushParam {
            branch: BranchName::main(),
            traces: Vec::with_capacity(0),
            compressed_objs: Vec::with_capacity(0),
            head: head.clone(),
        };
        save.execute(push_param).unwrap();
        let actual = mock.try_read(".meltos/branches/main/HEAD").unwrap();
        assert_eq!(actual, head.encode().unwrap());
    }
}