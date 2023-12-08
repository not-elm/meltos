use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error;

pub mod compression;
pub mod std_fs;


#[async_trait::async_trait]
pub trait FileIo: Send + Sync {
    async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result<FileData>;
    async fn write_file(&self, file_data: FileData) -> error::Result;
    async fn dir_entry_names<P: AsRef<Path> + Send + Sync>(
        &self,
        path: &P,
    ) -> crate::error::Result<Vec<PathBuf>>;

    async fn create_dir<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result;


    async fn read_dir<P: AsRef<Path> + Send + Sync>(
        &self,
        path: P,
    ) -> crate::error::Result<DirData> {
        let entry_names = self.dir_entry_names(&path).await?;
        let mut entries = Vec::with_capacity(entry_names.len());
        for entry_path in entry_names {
            entries.push(self.read_entry(entry_path).await?);
        }

        Ok(DirData {
            path: path.as_ref().to_str().unwrap().to_string(),
            children: entries,
        })
    }


    async fn read_entry<P: AsRef<Path> + Send + Sync>(
        &self,
        path: P,
    ) -> crate::error::Result<EntryData> {
        if path.as_ref().is_dir() {
            Ok(EntryData::Dir(self.read_dir(path).await?))
        } else {
            Ok(EntryData::File(self.read_file(path).await?))
        }
    }

    async fn write_entry(&self, entry: EntryData) -> crate::error::Result {
        match entry {
            EntryData::File(file) => self.write_file(file).await,
            EntryData::Dir(dir) => {
                self.create_dir(dir.path).await?;
                for entry in dir.children {
                    self.write_entry(entry).await?;
                }
                Ok(())
            }
        }
    }
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum EntryData {
    File(FileData),
    Dir(DirData),
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FileData {
    pub path: String,

    #[serde(with = "serde_bytes")]
    pub buf: Vec<u8>,
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DirData {
    pub path: String,

    pub children: Vec<EntryData>,
}
