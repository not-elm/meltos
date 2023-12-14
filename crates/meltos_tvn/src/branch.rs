use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

use crate::error;
use crate::file_system::{FileSystem, FsIo};
use crate::io::atomic::head::{HeadIo};
use crate::io::atomic::object::ObjectIo;
use crate::io::atomic::staging::StagingIo;
use crate::io::atomic::trace::TraceIo;
use crate::io::atomic::workspace::WorkspaceIo;
use crate::object::tree::Tree;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BranchName(pub String);
impl_string_new_type!(BranchName);

impl BranchName {
    #[inline]
    pub fn main() -> Self {
        Self::from("main")
    }
}

#[derive(Debug, Clone)]
pub struct BranchIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub(crate) now: TraceIo<Fs, Io>,
    stage: StagingIo<Fs, Io>,
    object: ObjectIo<Fs, Io>,
    workspace: WorkspaceIo<Fs, Io>,
    commit: HeadIo<Fs, Io>,
    branch_name: BranchName,
}

impl<Fs, Io> BranchIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new_main(fs: Fs) -> BranchIo<Fs, Io> {
        Self::new(BranchName::main(), fs)
    }

    pub fn new(branch_name: BranchName, fs: Fs) -> BranchIo<Fs, Io> {
        Self {
            object: ObjectIo::new(fs.clone()),
            stage: StagingIo::new(fs.clone()),
            workspace: WorkspaceIo(FsIo::new(fs.clone())),
            now: TraceIo::new(branch_name.clone(), fs.clone()),
            commit: HeadIo::new(branch_name.clone(), fs),
            branch_name,
        }
    }
}

impl<Fs, Io> BranchIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn init(&self) -> error::Result {
        if self.now.exists()? {
            return Err(error::Error::BranchAlreadyInitialized(
                self.branch_name.clone(),
            ));
        }
        let mut now_tree = Tree::default();
        for meta in self.workspace.convert_to_objs(".")? {
            let meta = meta?;
            self.object.write(&meta.obj)?;
            now_tree.insert(meta.file_path, meta.obj.hash);
        }

        let now_obj = now_tree.as_obj()?;
        self.now.write_hash(&now_obj.hash)?;
        self.object.write(&now_obj)?;
        Ok(())
    }

    pub fn unpack_project(&self) -> error::Result {
        let Some(now_tree) = self.read_now_tree()? else {
            return Ok(());
        };
        for (path, hash) in now_tree.iter() {
            let obj = self.object.try_read_obj(hash)?;
            self.workspace.unpack(path, &obj.buf)?;
        }
        Ok(())
    }

   


    fn read_now_tree(&self) -> error::Result<Option<Tree>> {
        let Some(now_obj_hash) = self.now.read_hash()? else {
            return Ok(None);
        };
        Ok(Some(self.object.read_to_tree(&now_obj_hash)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchIo;
    use crate::file_system::FileSystem;
    use crate::file_system::mock::MockFileSystem;
    use crate::object::ObjectHash;

    #[test]
    fn init() {
        let mock = MockFileSystem::default();
        mock.write_all("./src/main.rs", b"bdadasjlgd").unwrap();
        mock.write_all("./test.rs", b"test").unwrap();
        let branch = BranchIo::new_main(mock.clone());
        branch.init().unwrap();

        assert!(&mock
            .read_to_end(&format!(
                ".meltos/objects/{}",
                ObjectHash::new(b"bdadasjlgd")
            ))
            .is_ok());
        assert!(&mock
            .read_to_end(&format!(".meltos/objects/{}", ObjectHash::new(b"test")))
            .is_ok());
        assert!(&mock.read_to_end(".meltos/branches/main/NOW").is_ok());
    }

    #[test]
    fn failed_init_if_has_been_initialized() {
        let mock = MockFileSystem::default();
        let branch = BranchIo::new_main(mock.clone());
        branch.init().unwrap();
        assert!(branch.init().is_err());
    }

    #[test]
    fn create_stage_file_after_staged() {
        // let mock = MockFileSystem::default();
        // mock.write_all("./src/main.rs", b"fn main(){println(\"hello\")}")
        //     .unwrap();
        // mock.write_all("./src/test.rs", b"test").unwrap();
        // let branch = BranchIo::new_main(mock);
        // branch.stage("./src").unwrap();
        // let stage = branch.stage.read_tree().unwrap().unwrap();
        // assert_eq!(
        //     stage.get(&FilePath::from_path("./src/main.rs")),
        //     Some(&ObjectHash::new(b"fn main(){println(\"hello\")}"))
        // );
        // assert_eq!(
        //     stage.get(&FilePath::from_path("./src/test.rs")),
        //     Some(&ObjectHash::new(b"test"))
        // );
        todo!();
    }

    #[test]
    fn create_objs_after_staged() {
        // let mock = MockFileSystem::default();
        // mock.write_all("./src/main.rs", b"fn main(){println(\"hello\")}")
        //     .unwrap();
        // mock.write_all("./src/test.rs", b"test").unwrap();
        // let branch = BranchIo::new_main(mock.clone());
        // branch.stage("./src").unwrap();
        // let hash1 = ObjectHash::new(b"fn main(){println(\"hello\")}");
        // let hash2 = ObjectHash::new(b"test");
        // assert!(mock
        //     .read_to_end(&FilePath(format!(".meltos/objects/{hash1}")))
        //     .is_ok());
        // assert!(mock
        //     .read_to_end(&FilePath(format!(".meltos/objects/{hash2}")))
        //     .is_ok());
        todo!();
    }
}
