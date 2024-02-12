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
            staging: StagingIo::new(fs),
        }
    }

    pub async fn execute(&self, file_path: &str) -> error::Result {
        if let Some(mut staging) = self.staging.read().await? {
            staging.remove(&FilePath::from_path(file_path));
            self.staging.write_tree(&staging).await?;
        }
        Ok(())
    }

    #[inline(always)]
    pub async fn execute_all(&self) -> error::Result {
        self.staging.write_tree(&TreeObj::default()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::branch::BranchName;
    use crate::file_system::memory::MemoryFileSystem;
    use crate::file_system::FilePath;
    use crate::io::atomic::staging::StagingIo;
    use crate::object::tree::TreeObj;
    use crate::operation::init::Init;
    use crate::operation::stage::Stage;
    use crate::operation::un_stage::UnStage;

    #[tokio::test]
    async fn deleted_hello_from_staging() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        let stage = Stage::new(fs.clone());
        let un_stage = UnStage::new(fs.clone());
        let staging = StagingIo::new(fs.clone());
        let file_path = "hello.txt";

        init.execute(&branch).await.unwrap();
        fs.write_sync(file_path, b"hello");

        stage.execute(&branch, file_path).await.unwrap();

        assert!(staging
            .read()
            .await
            .unwrap()
            .unwrap()
            .contains_key(&FilePath::from_path(file_path)));
        un_stage.execute(file_path).await.unwrap();
        assert!(!staging
            .read()
            .await
            .unwrap()
            .unwrap()
            .contains_key(&FilePath::from_path(file_path)));
    }

    #[tokio::test]
    async fn delete_all_files_from_staging() {
        let fs = MemoryFileSystem::default();
        let branch = BranchName::owner();
        let init = Init::new(fs.clone());
        let stage = Stage::new(fs.clone());
        let un_stage = UnStage::new(fs.clone());
        let staging = StagingIo::new(fs.clone());
        init.execute(&branch).await.unwrap();

        fs.write_sync("hello1.txt", b"hello");
        fs.write_sync("hello2.txt", b"hello");
        fs.write_sync("hello3.txt", b"hello");
        stage.execute(&branch, ".").await.unwrap();

        let current_stagings = staging.read().await.unwrap().unwrap();
        assert!(current_stagings.contains_key(&FilePath::from_path("hello1.txt")));
        assert!(current_stagings.contains_key(&FilePath::from_path("hello2.txt")));
        assert!(current_stagings.contains_key(&FilePath::from_path("hello3.txt")));

        un_stage.execute_all().await.unwrap();
        let current_stagings = staging.read().await.unwrap().unwrap();
        assert!(!current_stagings.contains_key(&FilePath::from_path("hello1.txt")));
        assert!(!current_stagings.contains_key(&FilePath::from_path("hello2.txt")));
        assert!(!current_stagings.contains_key(&FilePath::from_path("hello3.txt")));
        assert_eq!(current_stagings, TreeObj::default());
    }
}
