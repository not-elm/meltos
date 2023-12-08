use crate::fs::{FileData, FileIo};

use std::path::{Path, PathBuf};

pub struct StdFs;


#[async_trait::async_trait]
impl FileIo for StdFs {
    async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result<FileData> {
        Ok(FileData {
            path: path.as_ref().to_str().unwrap().to_string(),
            buf: std::fs::read(path)?,
        })
    }


    async fn write_file(&self, file_data: FileData) -> crate::error::Result {
        std::fs::write(file_data.path, file_data.buf)?;
        Ok(())
    }


    async fn dir_entry_names<P: AsRef<Path> + Send + Sync>(
        &self,
        path: &P,
    ) -> crate::error::Result<Vec<PathBuf>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| Some(entry.ok()?.path()))
            .collect())
    }


    async fn create_dir<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result {
        std::fs::create_dir_all(path)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::fs::std_fs::StdFs;
    use crate::fs::{FileData, FileIo};
    use crate::test_util::unwind;

    #[tokio::test]
    async fn read_dir() -> crate::error::Result {
        unwind("tests/test_std1", async {
            StdFs.create_dir("tests/sample1").await?;
            StdFs.read_dir("tests/sample1").await?;
            Ok(())
        })
        .await
    }


    #[tokio::test]
    async fn write_file() -> crate::error::Result {
        let path = "tests/test_std2.txt";
        unwind(path, async move {
            StdFs
                .write_file(FileData {
                    path: path.to_string(),
                    buf: "hello world!".as_bytes().to_vec(),
                })
                .await?;

            let data = StdFs.read_file(path).await?;
            assert_eq!(data.path, path);
            assert_eq!(data.buf, "hello world!".as_bytes());
            Ok(())
        })
        .await
    }
}
