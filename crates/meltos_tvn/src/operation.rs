use crate::branch::BranchName;
use crate::error;
use crate::file_system::FileSystem;
use crate::io::atomic::work_branch::WorkingIo;
use crate::operation::commit::Commit;
use crate::operation::init::Init;
use crate::operation::push::Push;
use crate::operation::stage::Stage;

pub mod init;
pub mod stage;
pub mod commit;
pub mod push;
pub mod unzip;


#[derive(Debug, Clone)]
pub struct Operations<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    pub init: Init<Fs, Io>,
    pub stage: Stage<Fs, Io>,
    pub commit: Commit<Fs, Io>,
    pub push: Push<Fs, Io>,
}


impl<Fs, Io> Operations<Fs, Io>
    where
        Fs: FileSystem<Io> + Clone,
        Io: std::io::Write + std::io::Read,
{
    #[inline]
    pub fn new_main(fs: Fs) -> Operations<Fs, Io> {
        Self::new(BranchName::main(), fs)
    }

    #[inline]
    pub fn new_work(fs: Fs) -> error::Result<Operations<Fs, Io>> {
        let work = WorkingIo::new(fs.clone());
        Ok(Self::new(work.read()?, fs))
    }

    pub fn new(branch_name: BranchName, fs: Fs) -> Operations<Fs, Io> {
        Self {
            init: Init::new(branch_name.clone(), fs.clone()),
            stage: Stage::new(branch_name.clone(), fs.clone()),
            commit: Commit::new(branch_name.clone(), fs.clone()),
            push: Push::new(branch_name, fs),
        }
    }
}


#[cfg(test)]
mod tests {
    // use crate::branch::BranchIo;
    // use crate::file_system::FileSystem;
    // use crate::file_system::mock::MockFileSystem;
    // use crate::object::ObjectHash;

    //
    // #[test]
    // fn create_stage_file_after_staged() {
    //     // let mock = MockFileSystem::default();
    //     // mock.write_all("./src/main.rs", b"fn main(){println(\"hello\")}")
    //     //     .unwrap();
    //     // mock.write_all("./src/test.rs", b"test").unwrap();
    //     // let branch = BranchIo::new_main(mock);
    //     // branch.stage("./src").unwrap();
    //     // let stage = branch.stage.read_tree().unwrap().unwrap();
    //     // assert_eq!(
    //     //     stage.get(&FilePath::from_path("./src/main.rs")),
    //     //     Some(&ObjectHash::new(b"fn main(){println(\"hello\")}"))
    //     // );
    //     // assert_eq!(
    //     //     stage.get(&FilePath::from_path("./src/test.rs")),
    //     //     Some(&ObjectHash::new(b"test"))
    //     // );
    //     todo!();
    // }
    //
    // #[test]
    // fn create_objs_after_staged() {
    //     // let mock = MockFileSystem::default();
    //     // mock.write_all("./src/main.rs", b"fn main(){println(\"hello\")}")
    //     //     .unwrap();
    //     // mock.write_all("./src/test.rs", b"test").unwrap();
    //     // let branch = BranchIo::new_main(mock.clone());
    //     // branch.stage("./src").unwrap();
    //     // let hash1 = ObjectHash::new(b"fn main(){println(\"hello\")}");
    //     // let hash2 = ObjectHash::new(b"test");
    //     // assert!(mock
    //     //     .read_to_end(&FilePath(format!(".meltos/objects/{hash1}")))
    //     //     .is_ok());
    //     // assert!(mock
    //     //     .read_to_end(&FilePath(format!(".meltos/objects/{hash2}")))
    //     //     .is_ok());
    //     todo!();
    // }
}
