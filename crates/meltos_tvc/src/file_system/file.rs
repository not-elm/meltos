use std::fs::{DirEntry, File};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::file_system::{FileSystem, Stat, StatType};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let path = Path::new(path);
        if !path.exists() {
            return Ok(None);
        }
        let meta_data = path.metadata()?;
        Ok(Some(Stat {
            ty: if meta_data.is_file() { StatType::File } else { StatType::Dir },
            create_time: meta_data.created()?.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            update_time: meta_data.modified()?.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            size: if meta_data.is_file() {meta_data.len()} else {
                std::fs::read_dir(path)?.collect::<Vec<std::io::Result<DirEntry>>>().len() as u64
            }
        }))
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
        std::fs::create_dir_all(path)
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        match File::open(path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                Ok(Some(buf))
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        if !Path::new(path).exists() {
            return Ok(None);
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry.path().to_str().unwrap().to_string());
        }
        Ok(Some(entries))
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


#[cfg(test)]
mod tests {
    use crate::file_system::{FileSystem, StatType};
    use crate::file_system::file::StdFileSystem;

    fn tmp_dir() -> String {
        let path = directories::BaseDirs::new().unwrap().data_local_dir().to_path_buf();
        path.join("meltos_tmp").to_str().unwrap().to_string()
    }


    fn as_path(path: &str) -> String {
        format!("{}/{path}", tmp_dir())
    }


    #[test]
    fn return_none_if_not_exists() {
        let fs = StdFileSystem;
        let path = as_path("dir");
        fs.delete(&path).unwrap();

        assert!(fs.read_dir(&as_path("dir")).unwrap().is_none());
    }

    #[test]
    fn create_dir() {
        let fs = StdFileSystem;
        fs.create_dir(&as_path("dir")).unwrap();
        assert_eq!(fs.read_dir(&as_path("dir")).unwrap().unwrap().len(), 0);
    }


    #[test]
    fn create_parent_dirs_when_write_file() {
        let fs = StdFileSystem;
        let path = as_path("dir/hello.txt");
        fs.delete(&path).unwrap();
        fs.write_file(&path, b"hello").unwrap();
        assert_eq!(fs.read_file(&path).unwrap().unwrap(), b"hello");
    }


    #[test]
    fn delete_file() {
        let fs = StdFileSystem;
        let path = as_path("dir/hello.txt");
        fs.write_file(&path, b"hello").unwrap();
        fs.delete(&path).unwrap();
        assert!(fs.read_file(&path).unwrap().is_none());
    }


    #[test]
    fn stat_file() {
        let fs = StdFileSystem;
        let path = as_path("dir/hello.txt");
        fs.write_file(&path, b"hello").unwrap();
        let stat = fs.stat(&path).unwrap().unwrap();
        assert_eq!(stat.ty, StatType::File);
        assert_eq!(stat.size, b"hello".len() as u64);
    }


    #[test]
    fn stat_dir() {
        let fs = StdFileSystem;
        let path = as_path("dir");
        fs.delete(&as_path("dir")).unwrap();
        fs.create_dir(&as_path("dir/sample")).unwrap();

        let stat = fs.stat(&as_path("dir")).unwrap().unwrap();
        assert_eq!(stat.ty, StatType::Dir);
        assert_eq!(stat.size, 1);

        fs.write_file(&as_path("dir/hello.txt"), b"hello").unwrap();
        let stat = fs.stat(&path).unwrap().unwrap();
        assert_eq!(stat.ty, StatType::Dir);
        assert_eq!(stat.size, 2);
    }
}