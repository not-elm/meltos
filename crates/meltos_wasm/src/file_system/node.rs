use std::path::Path;

use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{Object, Uint8Array};

use meltos_tvc::file_system::{FileSystem, Stat, StatType};
use meltos_util::path::AsUri;


use crate::file_system::node::fs::{exists_sync, is_file, lstat_sync, mkdir_sync, read_dir_sync, read_file_sync, rm_dir_sync, rm_sync, write_file_sync};

mod buffer;
mod error;
mod stats;
mod fs;

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct NodeFileSystem {
    pub workspace_folder: String,
}

#[wasm_bindgen]
impl NodeFileSystem {
    #[wasm_bindgen(constructor)]
    pub fn new(workspace_folder: String) -> Self {
        Self {
            workspace_folder,
        }
    }

    #[inline]
    pub fn path(&self, path: &str) -> String {
        if path == "./" || path == "." {
            self.workspace_folder.clone()
        } else if path.starts_with(&self.workspace_folder) {
            path.to_string()
        } else {
            format!("{}/{}", self.workspace_folder, path.replace("./", ""))
        }
    }

    // #[inline(always)]
    // pub async fn create_dir_api(&self, path: &str) -> JsResult<()> {
    //     self.create_dir(path).await.into_js_result()
    // }
    //
    // #[inline(always)]
    // pub async fn write_file_api(&self, path: &str, buf: Vec<u8>) -> JsResult<()> {
    //     self.write_file(path, &buf).await.into_js_result()
    // }
    //
    // #[inline(always)]
    // pub async fn read_dir_api(&self, path: &str) -> JsResult<Option<Vec<String>>> {
    //     self.read_dir(path).await.into_js_result()
    // }
    //
    // #[inline(always)]
    // pub async fn read_file_api(&self, path: &str) -> JsResult<Option<Vec<u8>>> {
    //     self.read_file(path).await.into_js_result()
    // }
    //
    // #[inline(always)]
    // pub async fn delete_api(&self, path: &str) -> JsResult<()> {
    //     self.delete(path).await.into_js_result()
    // }
    //
    // #[inline(always)]
    // pub async fn stat_api(&self, path: &str) -> JsResult<Option<Stat>> {
    //     Ok(self.stat(path).await.into_js_result()?);
    // }
}

impl NodeFileSystem {
    pub fn write_sync(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let path = self.path(path);
        if let Some(dir) = Path::new(&path).parent() {
            let dir = dir.as_uri();
            self.create_dir_sync(&dir)?;
        }
        write_file_sync(&path, buf.to_vec(), &Object::new());
        Ok(())
    }


    pub fn create_dir_sync(&self, path: &str) -> std::io::Result<()> {
        let path = &self.path(path);
        if exists_sync(path)? {
            return Ok(());
        }
        match mkdir_sync(
            path,
            MkdirOptions {
                recursive: true,
            },
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.already_exists() {
                    Ok(())
                } else {
                    Err(std::io::Error::other(format!("failed to create dir {e:?}")))
                }
            }
        }
    }

    fn read_files(&self, files: &mut Vec<String>, path: String) -> std::io::Result<()> {
        if !exists_sync(&path)? {
            return Ok(());
        } else if is_file(&path)? {
            files.push(path.trim_start_matches(&self.workspace_folder).to_string());
        } else if let Some(entries) = read_dir_sync(&path)? {
            for entry in entries {
                self.read_files(files, format!("{path}/{entry}"))?;
            }
        }
        Ok(())
    }
}

fn rm_recursive(path: String) -> std::io::Result<()> {
    if !exists_sync(&path)? {
        Ok(())
    } else if is_file(&path)? {
        rm_sync(&path)
    } else if let Some(entries) = read_dir_sync(&path)? {
        for entry in entries {
            rm_recursive(format!("{path}/{entry}"))?;
        }
        rm_dir_sync(&path).map_err(|e| std::io::Error::other(format!("failed fs.rmdirSync : {e:?}")))?;
        Ok(())
    } else {
        Ok(())
    }
}

#[wasm_bindgen]
#[derive(serde::Serialize, serde::Deserialize)]
struct MkdirOptions {
    pub recursive: bool,
}


#[async_trait(? Send)]
impl FileSystem for NodeFileSystem {
    async fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let stats = lstat_sync(&path)?;
            Ok(Some(Stat {
                ty: if stats.is_file() {
                    StatType::File
                } else {
                    StatType::Dir
                },
                size: if stats.is_file() {
                    stats.size() as u64
                } else {
                    read_dir_sync(&path)?.unwrap_or_default().len() as u64
                },
                create_time: (stats.c_time_ms() / 1000) as u64,
                update_time: (stats.m_time_ms() / 1000) as u64,
            }))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    async fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.write_sync(path, buf)
    }

    #[inline(always)]
    async fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.create_dir_sync(path)
    }

    async fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let buffer = read_file_sync(&path)
                .map_err(|e| std::io::Error::other(format!("failed read file: {e:?}")))?;

            if buffer.is_string() {
                Ok(Some(buffer.as_string().unwrap().into_bytes()))
            } else {
                let buffer: Uint8Array = buffer.unchecked_into();
                let buffer = buffer.to_vec();
                Ok(Some(buffer))
            }
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    async fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        read_dir_sync(&self.path(path))
    }

    #[inline(always)]
    async fn delete(&self, path: &str) -> std::io::Result<()> {
        let entry_path = self.path(path);
        rm_recursive(entry_path)
    }

    #[inline]
    async fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let mut files = Vec::new();
            self.read_files(&mut files, path)?;
            Ok(files)
        } else {
            Ok(Vec::with_capacity(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;

    use wasm_bindgen_test::wasm_bindgen_test;

    use meltos_tvc::file_system::FileSystem;

    use crate::tests::node_fs;

    #[wasm_bindgen_test]
    async fn read_root_dir() {
        let fs = node_fs();
        fs.create_dir_sync("dir1").unwrap();
        let dir = fs.read_dir("dir1").await.unwrap();
        assert_eq!(dir.unwrap().len(), 0);
    }

    #[wasm_bindgen_test]
    async fn read_root_dir_with_files() {
        let fs = node_fs();
        fs.write_file("dir2/1.txt", b"1").await.unwrap();
        fs.write_file("dir2/2.txt", b"1").await.unwrap();
        fs.write_file("dir2/3.txt", b"1").await.unwrap();

        let dir = fs.read_dir("dir2").await.unwrap();
        assert_eq!(dir.unwrap().len(), 3);
    }

    #[wasm_bindgen_test]
    async fn create_src_dir() {
        let fs = node_fs();
        fs.create_dir("/src").await.unwrap();
        let dir = fs.try_read_dir("/src").await.unwrap();
        assert_eq!(dir.len(), 0);

        fs.write_file("/src/hello.txt", b"hello").await.unwrap();
        let src = fs.try_read_dir("/src").await.unwrap();
        assert_eq!(src.len(), 1);
    }

    #[wasm_bindgen_test]
    async fn create_parent_dirs() {
        let fs = node_fs();
        fs.write_sync("/dist/hello.txt", b"hello");
        fs.write_sync("/dist/hello2.txt", b"hello");
        fs.write_sync("/dist/hello3.txt", b"hello");

        let dist = fs.try_read_dir("/dist").await.unwrap();
        assert_eq!(dist.len(), 3);
    }

    #[wasm_bindgen_test]
    async fn read_hello_world() {
        let fs = node_fs();
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

    #[wasm_bindgen_test]
    async fn read_file_start_with_period() {
        let fs = node_fs();
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

    #[wasm_bindgen_test]
    async fn delete_file() {
        let fs = node_fs();
        fs.write_sync("hello.txt", b"hello world");
        fs.delete("hello.txt").await.unwrap();

        assert_eq!(fs.read_file("hello.txt").await.unwrap(), None);
    }

    #[wasm_bindgen_test]
    async fn delete_dir() {
        let fs = node_fs();
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

    #[wasm_bindgen_test]
    async fn all_files_with_in_children() {
        let fs = node_fs();
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

    #[wasm_bindgen_test]
    async fn all_files_recursive() {
        let fs = node_fs();
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

    #[wasm_bindgen_test]
    async fn all_files_relative_to_src() {
        let fs = node_fs();
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


    #[wasm_bindgen_test]
    async fn all_files_specified_direct_file_uri() {
        let fs = node_fs();
        fs.write_sync("/hello1.txt", b"hello");

        let files = fs.all_files_in("/hello1.txt").await.unwrap();
        assert_eq!(files, vec!["/hello1.txt".to_string()]);
    }

    #[wasm_bindgen_test]
    async fn return_none_if_not_exists_entry() {
        let fs = node_fs();
        fs.create_dir("src").await.unwrap();
        let stat = fs.stat("/hello.txt").await.unwrap();
        assert_eq!(stat, None);
        let stat = fs.stat("/src/hello.txt").await.unwrap();
        assert_eq!(stat, None);
    }

    #[wasm_bindgen_test]
    async fn stat_file() {
        let fs = node_fs();
        fs.write_file("src/hello.txt", b"hello").await.unwrap();
        let stat = fs.stat("src/hello.txt").await.unwrap().unwrap();
        assert!(stat.is_file());
        assert_eq!(stat.size, b"hello".len() as u64);
    }

    #[wasm_bindgen_test]
    async fn stat_dir() {
        let fs = node_fs();
        fs.create_dir("src").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert!(stat.is_dir());
        assert_eq!(stat.size, 0);
    }

    #[wasm_bindgen_test]
    async fn update_dir_stat() {
        let fs = node_fs();
        fs.create_dir("src").await.unwrap();

        fs.create_dir("src/dist").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert_eq!(stat.size, 1);

        fs.write_file("src/hello.txt", b"hello world").await.unwrap();
        let stat = fs.stat("src").await.unwrap().unwrap();
        assert_eq!(stat.size, 2);
    }

    #[wasm_bindgen_test]
    async fn update_file_stat() {
        let fs = node_fs();
        fs.write_file("src/hello.txt", b"hello world").await.unwrap();
        let stat1 = fs.stat("src/hello.txt").await.unwrap().unwrap();
        sleep(Duration::new(1, 100));
        fs.write_file("src/hello.txt", b"hello").await.unwrap();
        let stat2 = fs.stat("src/hello.txt").await.unwrap().unwrap();
        assert_eq!(stat1.create_time, stat2.create_time);
        assert_eq!(stat2.size, b"hello".len() as u64);

        assert!(stat1.update_time < stat2.update_time);
    }

    #[wasm_bindgen_test]
    async fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let fs = node_fs();

        fs.write_file("buf1", &buf1).await.unwrap();
        fs.write_file("buf2", &buf2).await.unwrap();
        assert_eq!(fs.read_file("buf1").await.unwrap().unwrap(), buf1.to_vec());
        assert_eq!(fs.read_file("buf2").await.unwrap().unwrap(), buf2.to_vec());
    }
}
