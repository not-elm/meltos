use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use crate::file_system::{FileSystem, Stat};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        todo!()
    }

    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let path: &Path = path.as_ref();
        if path.is_dir() {
            return Err(std::io::Error::other("path type should be file"));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        File::create(path)?.write_all(buf)
    }

    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        todo!()
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        println!("path={path}");
        match File::open(path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                println!("OK");
                Ok(Some(buf))
            }
            Err(error) => {
                println!("err={error}");
                if error.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        todo!()
    }

    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        if Path::new(path).is_dir() {
            let mut p = Vec::new();
            for entry in std::fs::read_dir(path)? {
                p.extend(self.all_files_in(entry?.path().to_str().unwrap())?);
            }
            Ok(p)
        } else if std::fs::File::open(path).is_ok() {
            Ok(vec![path.to_string()])
        } else {
            Ok(Vec::with_capacity(0))
        }
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        let path: &Path = path.as_ref();
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

}
