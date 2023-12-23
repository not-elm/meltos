use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meltos_util::impl_string_new_type;

pub mod file;
pub mod mock;

pub trait FileSystem {
    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()>;

    fn read(&self, path: &str) -> std::io::Result<Option<Vec<u8>>>;

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>>;

    fn delete(&self, path: &str) -> std::io::Result<()>;

    fn all_workspace_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        Ok(self
            .all_file_path(path)?
            .into_iter()
            .filter(|p| !p.contains(".meltos"))
            .collect())
    }

    fn try_read(&self, path: &str) -> std::io::Result<Vec<u8>> {
        self.read(path).and_then(|buf| {
            match buf {
                Some(buf) => Ok(buf),
                None => Err(std::io::Error::new(ErrorKind::NotFound, "file not found")),
            }
        })
    }


    fn project_already_initialized(&self) -> std::io::Result<bool> {
        let files = self.all_file_path("./.meltos")?;
        Ok(!files.is_empty())
    }
}


#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct FilePath(pub String);
impl_string_new_type!(FilePath);

impl FilePath {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().to_str().unwrap().to_string())
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<String> for FilePath {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
