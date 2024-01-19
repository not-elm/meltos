use crate::error;
use crate::file_system::{FilePath, FileSystem};
use crate::io::atomic::staging::StagingIo;
use crate::object::tree::TreeObj;

#[derive(Debug, Clone)]
pub struct UnStage<Fs: FileSystem> {
    staging: StagingIo<Fs>,
}


impl<Fs: FileSystem> UnStage<Fs> {
    #[inline]
    pub fn new(fs: Fs) -> UnStage<Fs> {
        Self {
            staging: StagingIo::new(fs)
        }
    }


    pub fn execute(&self, file_path: &str) -> error::Result {
        if let Some(mut staging) = self.staging.read()? {
            staging.remove(&FilePath::from_path(file_path));
            self.staging.write_tree(&staging)?;
        }
        Ok(())
    }


    #[inline(always)]
    pub fn execute_all(&self) -> error::Result {
        self.staging.write_tree(&TreeObj::default())?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::FilePath;
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::staging::StagingIo;
    use crate::object::tree::TreeObj;
    use crate::operation::init::Init;
    use crate::operation::stage::Stage;
    use crate::operation::un_stage::UnStage;

    #[test]
    fn deleted_hello_from_staging() {
        let fs = MockFileSystem::default();
        let init = Init::new(BranchName::owner(), fs.clone());
        let stage = Stage::new(BranchName::owner(), fs.clone());
        let un_stage = UnStage::new(fs.clone());
        let staging = StagingIo::new(fs.clone());
        let file_path = "workspace/hello.txt";

        init.execute().unwrap();
        fs.force_write(file_path, b"hello");
        println!("{fs:?}");
        stage.execute(file_path).unwrap();

        assert!(staging.read().unwrap().unwrap().contains_key(&FilePath::from_path(file_path)));
        un_stage.execute(file_path).unwrap();
        assert!(!staging.read().unwrap().unwrap().contains_key(&FilePath::from_path(file_path)));
    }

    #[test]
    fn delete_all_files_from_staging() {
        let fs = MockFileSystem::default();
        let init = Init::new(BranchName::owner(), fs.clone());
        let stage = Stage::new(BranchName::owner(), fs.clone());
        let un_stage = UnStage::new(fs.clone());
        let staging = StagingIo::new(fs.clone());
        init.execute().unwrap();

        fs.force_write("workspace/hello1.txt", b"hello");
        fs.force_write("workspace/hello2.txt", b"hello");
        fs.force_write("workspace/hello3.txt", b"hello");
        stage.execute(".").unwrap();

        let current_stagings = staging.read().unwrap().unwrap();
        assert!(current_stagings.contains_key(&FilePath::from_path("workspace/hello1.txt")));
        assert!(current_stagings.contains_key(&FilePath::from_path("workspace/hello2.txt")));
        assert!(current_stagings.contains_key(&FilePath::from_path("workspace/hello3.txt")));

        un_stage.execute_all().unwrap();
        let current_stagings = staging.read().unwrap().unwrap();
        assert!(!current_stagings.contains_key(&FilePath::from_path("workspace/hello1.txt")));
        assert!(!current_stagings.contains_key(&FilePath::from_path("workspace/hello2.txt")));
        assert!(!current_stagings.contains_key(&FilePath::from_path("workspace/hello3.txt")));
        assert_eq!(current_stagings, TreeObj::default());
    }
}


