use async_trait::async_trait;

use crate::file_system::{FileSystem, Stat};

pub struct ClipPathFileSystem<Fs> {
    fs: Fs,
    clip_paths: Vec<String>,
}


impl<Fs: FileSystem> ClipPathFileSystem<Fs> {
    pub const fn new(fs: Fs, clip_paths: Vec<String>) -> ClipPathFileSystem<Fs> {
        Self {
            fs,
            clip_paths,
        }
    }


        fn restore_path(&self, path: &str, p: &str) -> String {
        if let Some(prefix) = self
            .clip_paths
            .iter()
            .find(|p| path.starts_with(p.as_str())) {
            format!("{prefix}{}", p.trim_start_matches(prefix))
        } else {
            p.to_string()
        }
    }

    fn path<'a>(&self, path: &'a str) -> &'a str {
        if let Some(prefix) = self
            .clip_paths
            .iter()
            .find(|p| path.starts_with(p.as_str())) {
            path.trim_start_matches(prefix)
        } else {
            path
        }
    }
}


#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
impl<Fs: FileSystem> FileSystem for ClipPathFileSystem<Fs> {
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        self.fs.stat(self.path(path)).await
    }

    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.fs.write_file(self.path(path), buf).await
    }

    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.fs.create_dir(self.path(path)).await
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        self.fs.read_file(self.path(path)).await
    }

    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let Some(entries) = self.fs.read_dir(self.path(path)).await? else{
            return Ok(None);
        };
        Ok(Some(entries
            .into_iter()
            .map(|p|self.restore_path(path, &p))
            .collect()
        ))
    }

    async fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(self.path(path)).await
    }
}

