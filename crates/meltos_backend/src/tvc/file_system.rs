use std::path::PathBuf;
use async_trait::async_trait;
use axum::extract::Path;
use meltos::room::RoomId;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::file_system::{FileSystem, Stat};

use crate::path::room_resource_dir;

#[derive(Debug, Clone)]
pub struct BackendFileSystem<Fs = StdFileSystem> {
    room_id: RoomId,
    fs: Fs,
}

impl<Fs> BackendFileSystem<Fs> {
    #[inline]
    pub const fn new(room_id: RoomId, fs: Fs) -> BackendFileSystem<Fs> {
        Self {
            room_id,
            fs,
        }
    }

    #[inline(always)]
    fn trim(&self, path: String) -> String {
println!("TR path={path}");
        path.trim_start_matches(&format!("/{}", room_resource_dir(&self.room_id)
                .to_str()
                .unwrap()
                .replace('\\', "/")))

        .to_string()
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        let dir = room_resource_dir(&self.room_id);
        let uri = dir.join(path.trim_start_matches("/")).to_str().unwrap().replace('\\', "/").to_string();
        println!("dad path={path} {uri}");
        uri
    }
}


#[async_trait]
impl<Fs: FileSystem> FileSystem for BackendFileSystem<Fs> {
    #[inline(always)]
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        self.fs.stat(&self.as_path(path)).await
    }

    #[inline(always)]
    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.fs.write_file(&self.as_path(path), buf).await
    }

    #[inline(always)]
    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.fs.create_dir(&self.as_path(path)).await
    }

    #[inline(always)]
    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        self.fs.read_file(&self.as_path(path)).await
    }

    #[inline(always)]
    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        Ok(self
            .fs
            .read_dir(&self.as_path(path))
            .await?
            .map(|files| files.into_iter().map(|path| self.trim(path)).collect()))
    }

    #[inline(always)]
    async fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        Ok(self
            .fs
            .all_files_in(&self.as_path(path))
            .await?
            .into_iter()
            .map(|file| self.trim(file))
            .collect())
    }

    #[inline(always)]
    async fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(&self.as_path(path)).await
    }
}

#[cfg(test)]
mod tests {
    use meltos::room::RoomId;
    use meltos_tvc::file_system::mock::MockFileSystem;
    use meltos_tvc::file_system::FileSystem;

    use crate::tvc::file_system::BackendFileSystem;

    #[tokio::test]
    async fn read_files_in_dir() {
        let fs = MockFileSystem::default();
        let fs = BackendFileSystem::new(RoomId::new(), fs.clone());
        fs.write_file("/dir/hello.txt", b"hello").await.unwrap();
        fs.write_file("/hello2.txt", b"hello").await.unwrap();
        let mut files = fs.all_files_in(".").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec!["/dir/hello.txt".to_string(), "/hello2.txt".to_string(),]
        )
    }
}
