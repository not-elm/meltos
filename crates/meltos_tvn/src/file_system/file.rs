use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::file_system::FileSystem;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StdFileSystem;

impl FileSystem<File> for StdFileSystem {
    fn open_file(&self, path: &str) -> std::io::Result<Option<File>> {
        match File::open(path) {
            Ok(file) => Ok(Some(file)),
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        if Path::new(path).is_dir() {
            let mut p = Vec::new();
            for entry in std::fs::read_dir(path)? {
                p.extend(self.all_file_path(entry?.path().to_str().unwrap())?);
            }
            Ok(p)
        } else if  std::fs::File::open(path).is_ok(){
            Ok(vec![path.to_string()])
        }else{
            Ok(Vec::with_capacity(0))
        }
    }

    #[inline(always)]
    fn create(&self, path: &str) -> std::io::Result<File> {
        let path: &Path = path.as_ref();
        if path.is_dir() {
            return Err(std::io::Error::other("path type should be file"));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        File::create(path)
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        let path: &Path = path.as_ref();
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }
}
