use std::path::{Path, PathBuf};

use crate::error::MelResult;
use crate::fs::{FileData, FsAccessible};

pub struct StdFs;


#[async_trait::async_trait]
impl FsAccessible for StdFs {
    async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> MelResult<FileData> {
        Ok(FileData {
            path: path.as_ref().to_str().unwrap().to_string(),
            buf: std::fs::read(path)?,
        })
    }


    async fn write_file(&self, file_data: FileData) -> MelResult {
        std::fs::write(file_data.path, file_data.buf)?;
        Ok(())
    }


    async fn dir_entry_names<P: AsRef<Path> + Send + Sync>(&self, path: &P) -> MelResult<Vec<PathBuf>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| { Some(entry.ok()?.path()) })
            .collect())
    }


    async fn create_dir<P: AsRef<Path> + Send>(&self, path: P) -> MelResult {
        std::fs::create_dir_all(path)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::error::MelResult;
    use crate::fs::{FileData, FsAccessible};
    use crate::fs::std_fs::StdFs;
    use crate::test_util::{create_tests_dir, unwind};

    #[tokio::test]
    async fn read_dir() -> MelResult {
        unwind(async {
            StdFs.create_dir("tests/sample1").await?;
            StdFs.read_dir("tests/sample1").await?;
            Ok(())
        })
            .await
    }


    #[tokio::test]
    async fn read_file() -> MelResult {
        unwind(async move {
            create_tests_dir()?;
            let data = StdFs.read_file("./tests/hello.txt").await?;
            assert_eq!(data.path, "./tests/hello.txt");
            assert_eq!(data.buf, "hello".as_bytes());
            Ok(())
        }).await
    }


    #[tokio::test]
    async fn write_file() -> MelResult {
        unwind(async move {
            StdFs.write_file(FileData {
                path: "tests/test".to_string(),
                buf: "hello world!".as_bytes().to_vec(),
            }).await?;

            let data = StdFs.read_file("./tests/test").await?;
            assert_eq!(data.path, "./tests/test");
            assert_eq!(data.buf, "hello world!".as_bytes());
            Ok(())
        }).await
    }
}