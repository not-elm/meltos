use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::object::ObjIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::object::tree::TreeObj;

pub struct Init<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    branch_name: BranchName,
    workspace: WorkspaceIo<Fs, Io>,
    trace: TraceIo<Fs, Io>,
    object: ObjIo<Fs, Io>,
}


impl<Fs, Io> Init<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read
{
    pub fn new(branch_name: BranchName, fs: Fs) -> Init<Fs, Io> {
        Self {
            workspace: WorkspaceIo::new(fs.clone()),
            trace: TraceIo::new(branch_name.clone(), fs.clone()),
            object: ObjIo::new(fs.clone()),
            branch_name,
        }
    }
}


impl<Fs, Io> Init<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read
{
    pub fn execute(&self) -> error::Result {
        if self.trace.exists()? {
            return Err(error::Error::BranchAlreadyInitialized(
                self.branch_name.clone(),
            ));
        }
        let mut trace_tree = TreeObj::default();
        for meta in self.workspace.convert_to_objs(".")? {
            let meta = meta?;
            self.object.write(&meta.obj)?;
            trace_tree.insert(meta.file_path, meta.obj.hash);
        }
        let trace_obj = trace_tree.as_obj()?;
        self.trace.write(&trace_obj.hash)?;
        self.object.write(&trace_obj)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::object::ObjHash;
    use crate::operation::init::Init;

    #[test]
    fn init() {
        let mock = MockFileSystem::default();
        mock.write("./src/main.rs", b"bdadasjlgd").unwrap();
        mock.write("./test.rs", b"test").unwrap();
        let init = Init::new(BranchName::main(), mock.clone());
        init.execute().unwrap();

        assert!(&mock
            .read(&format!(
                ".meltos/objects/{}",
                ObjHash::new(b"bdadasjlgd")
            ))
            .is_ok());
        assert!(&mock
            .read(&format!(".meltos/objects/{}", ObjHash::new(b"test")))
            .is_ok());
        assert!(&mock.read(".meltos/branches/main/NOW").is_ok());
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let mock = MockFileSystem::default();
        let init = Init::new(BranchName::main(), mock.clone());
        init.execute().unwrap();
        assert!(init.execute().is_err());
    }
}