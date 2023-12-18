use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::head::HeadIo;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::object::commit::CommitHash;
use crate::object::{CompressedBuf, ObjHash};
use crate::operation::push::PushParam;

#[derive(Debug, Clone)]
pub struct Save<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read,
{
    trace: TraceIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
    head: HeadIo<Fs, Io>,
}


impl<Fs, Io> Save<Fs, Io>
where
    Fs: FileSystem<Io> + Clone,
    Io: std::io::Write + std::io::Read,
{
    pub fn new(fs: Fs) -> Save<Fs, Io> {
        Self {
            trace: TraceIo::new(fs.clone()),
            object: ObjIo::new(fs.clone()),
            head: HeadIo::new(fs),
        }
    }


    /// * write objs.
    /// * write head.
    /// * write traces related to commits.
    pub fn execute(&self, push_param: PushParam) -> error::Result {
        self.write_objs(push_param.compressed_objs)?;
        self.write_head(&push_param.branch, &push_param.head)?;
        self.write_traces(push_param.traces)
    }


    fn write_objs(&self, objs: Vec<(ObjHash, CompressedBuf)>) -> error::Result {
        for (hash, buf) in objs {
            self.object.write(&hash, &buf)?;
        }

        Ok(())
    }

    fn write_head(&self, branch: &BranchName, head_hash: &CommitHash) -> error::Result {
        self.head.write(branch, head_hash)?;
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
        let actual = mock.try_read(".meltos/refs/heads/main").unwrap();
        assert_eq!(actual, head.encode().unwrap());
    }
}
