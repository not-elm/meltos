use std::path::Path;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::{Object, Uint8Array};

use meltos_tvc::file_system::{FileSystem, Stat, StatType};

type NodeFsResult<T = JsValue> = std::result::Result<T, Error>;

#[wasm_bindgen(module = "fs")]
extern {
    #[derive(Debug)]
    type Error;

    #[wasm_bindgen(method, getter)]
    fn code(this: &Error) -> String;
}


impl Error {
    #[inline(always)]
    pub fn already_exists(&self) -> bool {
        self.code() == "EEXIST"
    }


    #[inline(always)]
    pub fn not_found(&self) -> bool {
        self.code() == "ENOENT"
    }
}


#[wasm_bindgen(module = "fs")]
extern "C" {
    #[derive(Debug)]
    type Stats;

    #[wasm_bindgen(method, js_name = isFile)]
    fn is_file(this: &Stats) -> bool;

    #[wasm_bindgen(method, getter)]
    fn size(this: &Stats) -> usize;

    #[wasm_bindgen(method, getter, js_name = ctimeMs)]
    fn c_time_ms(this: &Stats) -> usize;

    #[wasm_bindgen(method, getter, js_name = mtimeMs)]
    fn m_time_ms(this: &Stats) -> usize;
}


#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    fn read_file_sync(path: &str) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = mkdirSync, catch)]
    fn mkdir_sync(path: &str, options: MkdirOptions) -> NodeFsResult<Option<String>>;

    #[wasm_bindgen(js_name = writeFileSync)]
    fn write_file_sync(path: &str, data: Vec<u8>, options: &JsValue);

    #[wasm_bindgen(js_name = readdirSync, catch)]
    fn _read_dir_sync(path: &str, options: JsValue) -> NodeFsResult<Vec<String>>;

    #[wasm_bindgen(js_name = rmdirSync, catch)]
    fn rm_dir_sync(path: &str, options: MkdirOptions) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = rmSync, catch)]
    fn _rm_sync(path: &str) -> NodeFsResult<JsValue>;

    #[wasm_bindgen(js_name = existsSync, catch)]
    fn _exists_sync(path: &str) -> NodeFsResult<bool>;

    #[wasm_bindgen(js_name = lstatSync, catch)]
    fn _lstat_sync(path: &str) -> NodeFsResult<Stats>;
}


fn exists_sync(path: &str) -> std::io::Result<bool> {
    match _exists_sync(path) {
        Ok(exists) => Ok(exists),
        Err(e) => Err(std::io::Error::other(format!("failed fs.existsSysnc: {e:?}")))
    }
}


fn read_dir_sync(path: &str) -> std::io::Result<Option<Vec<String>>> {
    match _read_dir_sync(path, JsValue::null()) {
        Ok(entries) => Ok(Some(entries)),
        Err(e) => Err(std::io::Error::other(format!("failed read dir: {e:?}")))
    }
}


#[inline(always)]
fn rm_sync(path: &str) -> std::io::Result<()> {
    _rm_sync(path).map_err(|e| {
        std::io::Error::other(format!("failed fs.rmSync : {e:?}"))
    })?;
    Ok(())
}


#[inline]
fn lstat_sync(path: &str) -> std::io::Result<Stats> {
    let stats = _lstat_sync(path).map_err(|e| std::io::Error::other(format!("failed lstat_sync : {e:?}")))?;
    Ok(stats)
}


#[inline]
fn is_file(path: &str) -> std::io::Result<bool> {
    Ok(lstat_sync(path)?.is_file())
}

#[wasm_bindgen]
extern "C" {
    type Buffer;

    #[wasm_bindgen(method, getter)]
    fn buffer(this: &Buffer) -> Uint8Array;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    fn length(this: &Buffer) -> u32;
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct NodeFileSystem {
    pub workspace_folder: String,
}

#[wasm_bindgen]
impl NodeFileSystem {
    #[wasm_bindgen(constructor)]
    #[inline(always)]
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
}

impl NodeFileSystem {
    fn read_files(&self, files: &mut Vec<String>, path: String) -> std::io::Result<()> {
        if !exists_sync(&path)? {
            return Ok(());
        } else if is_file(&path)? {
            files.push(path.trim_start_matches(&self.workspace_folder).to_string());
        } else if let Some(entries) = read_dir_sync(&path)? {
            for entry in entries {
                self.read_files(files, format!("{path}/{entry}").trim_end_matches("\n").to_string())?;
            }
        }
        Ok(())
    }
}


#[wasm_bindgen]
#[derive(serde::Serialize, serde::Deserialize)]
struct MkdirOptions {
    recursive: bool,
}

impl FileSystem for NodeFileSystem {
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let stats = lstat_sync(&path)?;
            Ok(Some(Stat {
                ty: if stats.is_file() { StatType::File } else { StatType::Dir },
                size: if stats.is_file() { stats.size() as u64 } else {
                    read_dir_sync(&path)?.unwrap_or_default().len() as u64
                },
                create_time: (stats.c_time_ms() / 1000) as u64,
                update_time: (stats.m_time_ms() / 1000) as u64,
            }))
        } else {
            Ok(None)
        }
    }

    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let path = self.path(path);
        if let Some(dir) = Path::new(&path).parent() {
            let dir = dir.to_str().unwrap();
            self.create_dir(dir)?;
        }
        write_file_sync(&path, buf.to_vec(), &Object::new());
        Ok(())
    }

    #[inline]
    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        let path = &self.path(path);
        if exists_sync(path)?{
            return Ok(());
        }
        match mkdir_sync(path, MkdirOptions {
            recursive: true
        }) {
            Ok(_) => Ok(()),
            Err(e) => if e.already_exists() {
                Ok(())
            } else {
                Err(std::io::Error::other(format!("failed to create dir {e:?}")))
            },
        }
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
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
    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        read_dir_sync(&self.path(path))
    }

    #[inline]
    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let path = self.path(path);
        if exists_sync(&path)? {
            let mut files = Vec::new();
            self.read_files(&mut files, path)?;
            Ok(files)
        } else {
            Ok(Vec::with_capacity(0))
        }
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        let entry_path = self.path(path);
        if !exists_sync(&entry_path)? {
            return Ok(());
        }

        if is_file(&entry_path)? {
            rm_sync(&entry_path)?;
            Ok(())
        } else {
            for file_path in self.all_files_in(path)? {
                rm_sync(&self.path(&file_path))?;
            }

            match rm_dir_sync(&entry_path, MkdirOptions {
                recursive: true
            }) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.not_found() {
                        Ok(())
                    } else {
                        Err(std::io::Error::other(format!("failed delete dir: {e:?}")))
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use meltos_tvc::file_system::FileSystem;

    use crate::file_system::node::NodeFileSystem;

    fn workspace_folder() -> String {
        "D://tmp".to_string()
    }


    fn node_fs() -> NodeFileSystem {
        NodeFileSystem::new(workspace_folder())
    }

    #[wasm_bindgen_test]
    fn create_dir() {
        let fs = node_fs();
        fs.create_dir("src1").unwrap();
        let dir = fs.read_dir("src1");
        fs.delete("src1").unwrap();
        assert!(dir.unwrap().is_some());
    }

    #[wasm_bindgen_test]
    fn success_if_already_created_dir() {
        let fs = node_fs();
        fs.create_dir("src2").unwrap();
        fs.create_dir("src2").unwrap();

        fs.delete("src2").unwrap();
    }


    #[wasm_bindgen_test]
    fn success_if_not_found() {
        let fs = node_fs();

        fs.delete("src3").unwrap();
        fs.delete("src3").unwrap();
    }


    #[wasm_bindgen_test]
    fn write_file_below_root() {
        let fs = node_fs();

        fs.write_file("hello.txt", b"hello").unwrap();
        let buf = fs.read_file("hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello".to_vec()));
        fs.delete("hello.txt").unwrap();
    }

    #[wasm_bindgen_test]
    fn write_file_below_src() {
        let fs = node_fs();
        fs.delete("src4").unwrap();
        fs.write_file("src4/hello.txt", b"hello").unwrap();
        let buf = fs.read_file("src4/hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello".to_vec()));

        fs.write_file("src4/hello2.txt", b"hello").unwrap();
        let buf = fs.read_file("src4/hello2.txt").unwrap();
        assert_eq!(buf, Some(b"hello".to_vec()));

        fs.delete("src4").unwrap();
    }


    #[wasm_bindgen_test]
    fn stat_file() {
        let fs = node_fs();
        fs.write_file("src5/hello.txt", b"hello").unwrap();
        let stat1 = fs.stat("src5/hello.txt").unwrap().unwrap();
        assert!(stat1.is_file());

        fs.write_file("src5/hello.txt", b"hello world!").unwrap();

        let stat2 = fs.stat("src5/hello.txt").unwrap().unwrap();
        assert_eq!(stat1.create_time, stat2.create_time);
        assert!(stat1.update_time <= stat2.update_time);
    }


    #[wasm_bindgen_test]
    fn stat_dir() {
        let fs = node_fs();
        fs.delete("src7").unwrap();
        fs.write_file("src7/hello.txt", b"hello").unwrap();
        let stat1 = fs.stat("src7").unwrap().unwrap();
        assert!(!stat1.is_file());
        fs.write_file("src7/hello2.txt", b"hello world!").unwrap();

        let stat2 = fs.stat("src7").unwrap().unwrap();
        assert_eq!(stat1.create_time, stat2.create_time);
        assert_eq!(stat1.size, 1);
        assert_eq!(stat2.size, 2);
        assert!(stat1.update_time <= stat2.update_time);
    }
}

