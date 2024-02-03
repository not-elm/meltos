use std::path::{Path, PathBuf, StripPrefixError};

use async_trait::async_trait;

use meltos_core::room::RoomId;
use meltos_tvc::file_system::std_fs::StdFileSystem;
use meltos_tvc::file_system::{FileSystem, Stat};
use meltos_util::path::AsUri;

use crate::path::room_resource_dir;

#[derive(Debug, Clone)]
pub struct BackendFileSystem<Fs = StdFileSystem> {
    fs: Fs,
    resource_dir_uri: PathBuf,
    resource_dir_uri_with_root: PathBuf,
}

impl<Fs> BackendFileSystem<Fs> {
    #[inline]
    pub fn new(room_id: RoomId, fs: Fs) -> BackendFileSystem<Fs> {
        Self {
            resource_dir_uri: room_resource_dir(&room_id),
            resource_dir_uri_with_root: room_resource_dir(&room_id).join("root"),
            fs,
        }
    }

    #[inline(always)]
    fn trim(&self, path: String) -> String {
        let p = Path::new(&path);
        let p = self.strip_resource_dir(p);
        p.map(|p| p.as_uri()).unwrap_or(path)
    }

    fn strip_resource_dir(&self, p: &Path) -> std::result::Result<PathBuf, StripPrefixError> {
        if let Ok(path) = p.strip_prefix(&self.resource_dir_uri_with_root) {
            Ok(Path::new("/").join(path))
        } else {
            Ok(p.strip_prefix(&self.resource_dir_uri)?.to_path_buf())
        }
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        let p = Path::new(path);

        let new_uri = if p.has_root() {
            self.resource_dir_uri_with_root
                .join(p.strip_prefix("/").unwrap())
        } else {
            self.resource_dir_uri.join(p)
        };

        new_uri.as_uri()
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
    async fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(&self.as_path(path)).await
    }
}

#[cfg(test)]
mod tests {
    use meltos_core::room::RoomId;
    use meltos_tvc::file_system::memory::MemoryFileSystem;
    use meltos_tvc::file_system::FileSystem;

    use crate::tvc::file_system::BackendFileSystem;

    #[tokio::test]
    async fn read_files_in_dir() {
        let fs = MemoryFileSystem::default();
        let fs = BackendFileSystem::new(RoomId::new(), fs.clone());
        fs.write_file("/dir/hello.txt", b"hello").await.unwrap();
        fs.write_file("/hello2.txt", b"hello").await.unwrap();
        let mut files = fs.all_files_in(".").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec!["/dir/hello.txt".to_string(), "/hello2.txt".to_string()]
        )
    }

    #[tokio::test]
    async fn read_files_without_root() {
        let fs = MemoryFileSystem::default();
        let fs = BackendFileSystem::new(RoomId::new(), fs.clone());
        fs.write_file("dir/hello.txt", b"hello").await.unwrap();
        fs.write_file("hello2.txt", b"hello").await.unwrap();
        let mut files = fs.all_files_in(".").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec!["dir/hello.txt".to_string(), "hello2.txt".to_string()]
        )
    }
}
