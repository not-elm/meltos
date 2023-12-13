use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::io::OpenIo;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FileOpen;


impl OpenIo<File> for FileOpen {
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
        if Path::new(path).is_dir(){
            let mut p = Vec::new();
            for entry in std::fs::read_dir(path)?{
                p.extend(self.all_file_path(entry?.path().to_str().unwrap())?);
            }
            Ok(p)
        }else{
            Ok(vec![path.to_string()])
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
}
