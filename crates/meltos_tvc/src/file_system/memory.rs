use std::fmt::Debug;
use std::path::Path;

use async_trait::async_trait;
use meltos_util::console_log;

use meltos_util::path::AsUri;

use crate::file_system::{FileSystem, Stat};
use crate::file_system::memory::entry::dir::MemoryDir;

mod entry;

#[derive(Clone, Debug)]
pub struct MemoryFileSystem(pub MemoryDir);


impl Default for MemoryFileSystem {
    #[inline(always)]
    fn default() -> Self {
        Self(MemoryDir::root())
    }
}

impl MemoryFileSystem {
    #[allow(unused)]
    pub fn write_sync(&self, path: &str, buf: &[u8]) {
        self.0.write_file(path, buf);
    }
}


#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl FileSystem for MemoryFileSystem {
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let Some(entry) = self.0.read(path) else {
            return Ok(None);
        };

        Ok(Some(entry.stat()))
    }

    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_file(path, buf);
        Ok(())
    }

    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.0.create_dir(path);
        Ok(())
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let Some(entry) = self.0.read(path) else {
            return Ok(None);
        };
        Ok(Some(entry.file()?.buf()))
    }

    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let Some(entry) = self.0.read(path) else {
            return Ok(None);
        };

        Ok(Some(entry.dir()?.entry_names()))
    }


    async fn delete(&self, path: &str) -> std::io::Result<()> {
        if let Some(parent) = self.0.lookup_parent_dir(path) {
            parent.delete(&entry_name(path));
        } else {
            self.0.delete(path);
        }
        Ok(())
    }
}

fn as_schemes(path: &str) -> Vec<String> {
    Path::new(path)
        .iter()
        .map(|name| name.to_str().unwrap())
        .map(|name| name.replace('\\', "/"))
        .collect::<Vec<String>>()
}

fn parent_path(path: &str) -> Option<String> {
    Some(Path::new(path).parent()?.as_uri())
}

fn entry_name(path: &str) -> String {
    let mut ps: Vec<&str> = path.split('/').collect();
    ps.pop().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;

    use crate::file_system::FileSystem;
    use crate::file_system::memory::MemoryFileSystem;

    #[tokio::test]
    async fn read_root_dir() {
        let fs = MemoryFileSystem::default();
        let dir = fs.read_dir(".").await.unwrap();
        assert_eq!(dir.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn read_root_dir_with_files() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("1.txt", b"1");
        fs.write_sync("2.txt", b"2");
        fs.write_sync("3.txt", b"3");

        let dir = fs.read_dir(".").await.unwrap();
        assert_eq!(dir.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn create_src_dir() {
        let fs = MemoryFileSystem::default();
        fs.create_dir("/src").await.unwrap();
        let dir = fs.try_read_dir("/src").await.unwrap();
        assert_eq!(dir.len(), 0);

        fs.write_sync("/src/hello.txt", b"hello");
        let src = fs.try_read_dir("/src").await.unwrap();
        assert_eq!(src.len(), 1);
    }

    #[tokio::test]
    async fn create_parent_dirs() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/dist/hello.txt", b"hello");
        fs.write_sync("/dist/hello2.txt", b"hello");
        fs.write_sync("/dist/hello3.txt", b"hello");

        let dist = fs.try_read_dir("/dist").await.unwrap();
        assert_eq!(dist.len(), 3);
    }

    #[tokio::test]
    async fn read_hello_world() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/hello.txt", b"hello world");
        fs.write_sync("/dist/hello.txt", b"hello world");
        fs.write_sync("/dist/sample/hello.txt", b"hello world");

        let buf = fs.read_file("/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("/dist/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("/dist/sample/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
    }

    #[tokio::test]
    async fn read_file_start_with_period() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("hello.txt", b"hello world");
        fs.write_sync("dist/hello.txt", b"hello world");
        fs.write_sync("dist/sample/hello.txt", b"hello world");

        let buf = fs.read_file("./hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("./dist/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = fs.read_file("./dist/sample/hello.txt").await.unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
    }

    #[tokio::test]
    async fn delete_file() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("hello.txt", b"hello world");
        fs.delete("hello.txt").await.unwrap();

        assert_eq!(fs.read_file("hello.txt").await.unwrap(), None);
    }

    #[tokio::test]
    async fn delete_dir() {
        let fs = MemoryFileSystem::default();
        fs.create_dir("src").await.unwrap();
        fs.write_file("src/hello.txt", b"hello").await.unwrap();

        fs.write_sync("dist/hello.txt", b"hello");
        fs.write_sync("dist/sample/sample.js", b"console.log(`sample`)");

        assert_eq!(fs.read_dir("src").await.unwrap().unwrap().len(), 1);
        assert_eq!(fs.read_dir("dist/sample").await.unwrap().unwrap().len(), 1);

        fs.delete("src").await.unwrap();
        assert!(fs.read_dir("src").await.unwrap().is_none());
        assert_eq!(fs.read_dir("dist/sample").await.unwrap().unwrap().len(), 1);
        assert_eq!(fs.try_read_dir("dist").await.unwrap().len(), 2);

        fs.delete("dist/sample").await.unwrap();
        assert!(fs.read_dir("src").await.unwrap().is_none());
        assert!(fs.read_dir("dist/sample").await.unwrap().is_none());
        assert_eq!(fs.try_read_dir("dist").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn all_files_with_in_children() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/hello1.txt", b"hello");
        fs.write_sync("/hello2.txt", b"hello");
        fs.write_sync("/hello3.txt", b"hello");

        let mut files = fs.all_files_in(".").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "/hello1.txt".to_string(),
                "/hello2.txt".to_string(),
                "/hello3.txt".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn all_files_recursive() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/hello1.txt", b"hello");
        fs.write_sync("/src/hello2.txt", b"hello");
        fs.write_sync("/src/dist/hello3.txt", b"hello");

        let mut files = fs.all_files_in(".").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "/hello1.txt".to_string(),
                "/src/dist/hello3.txt".to_string(),
                "/src/hello2.txt".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn all_files_relative_to_src() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/hello1.txt", b"hello");
        fs.write_sync("/src/hello2.txt", b"hello");
        fs.write_sync("/src/dist/hello3.txt", b"hello");

        let mut files = fs.all_files_in("/src").await.unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "/src/dist/hello3.txt".to_string(),
                "/src/hello2.txt".to_string(),
            ]
        );
    }


    #[tokio::test]
    async fn all_files_specified_direct_file_uri() {
        let fs = MemoryFileSystem::default();
        fs.write_sync("/hello1.txt", b"hello");

        let files = fs.all_files_in("/hello1.txt").await.unwrap();
        assert_eq!(files, vec!["/hello1.txt".to_string()]);
    }

    #[tokio::test]
    async fn return_none_if_not_exists_entry() {
        let fs = MemoryFileSystem::default();
        fs.create_dir("src").await.unwrap();
        let stat = fs.stat("/hello.txt").await.unwrap();
        assert_eq!(stat, None);
        let stat = fs.stat("/src/hello.txt").await.unwrap();
        assert_eq!(stat, None);
    }

    #[tokio::test]
    async fn stat_file() {
        let fs = MemoryFileSystem::default();
        fs.write_file("src/hello.txt", b"hello").await.unwrap();
        let stat = fs.stat("src/hello.txt").await.unwrap().unwrap();
        assert!(stat.is_file());
        assert_eq!(stat.size, b"hello".len() as u64);
    }

    #[tokio::test]
    async fn stat_dir() {
        let fs = MemoryFileSystem::default();
        fs.create_dir("src").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert!(stat.is_dir());
        assert_eq!(stat.size, 0);
    }

    #[tokio::test]
    async fn update_dir_stat() {
        let fs = MemoryFileSystem::default();
        fs.create_dir("src").await.unwrap();

        fs.create_dir("src/dist").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert_eq!(stat.size, 1);

        fs.write_file("src/hello.txt", b"hello world").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert_eq!(stat.size, 2);
    }

    #[tokio::test]
    async fn update_file_stat() {
        let fs = MemoryFileSystem::default();
        fs.write_file("src/hello.txt", b"hello world").await.unwrap();
        let stat1 = fs.stat("src/hello.txt").await.unwrap().unwrap();
        sleep(Duration::new(1, 100));
        fs.write_file("src/hello.txt", b"hello").await.unwrap();
        let stat2 = fs.stat("src/hello.txt").await.unwrap().unwrap();
        assert_eq!(stat1.create_time, stat2.create_time);
        assert_eq!(stat2.size, b"hello".len() as u64);

        assert!(stat1.update_time < stat2.update_time);
    }

    #[tokio::test]
    async fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let fs = MemoryFileSystem::default();

        fs.write_file("buf1", &buf1).await.unwrap();
        fs.write_file("buf2", &buf2).await.unwrap();
        assert_eq!(fs.read_file("buf1").await.unwrap().unwrap(), buf1.to_vec());
        assert_eq!(fs.read_file("buf2").await.unwrap().unwrap(), buf2.to_vec());
    }
}
