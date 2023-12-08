use std::path::{Path, PathBuf};

use crate::compression::CompressionBuf;
use crate::fs::{FileData, FileIo};

pub struct FsCompression<Comp, Fs> {
    comp: Comp,
    fs: Fs,
}


impl<Comp, Fs> FsCompression<Comp, Fs>
where
    Comp: CompressionBuf,
    Fs: FileIo,
{
    pub const fn new(comp: Comp, fs: Fs) -> FsCompression<Comp, Fs> {
        FsCompression { comp, fs }
    }
}


#[async_trait::async_trait]
impl<Comp, Fs> FileIo for FsCompression<Comp, Fs>
where
    Comp: CompressionBuf,
    Fs: FileIo,
{
    async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result<FileData> {
        let file = self.fs.read_file(path).await?;
        Ok(FileData {
            path: file.path,
            buf: self.comp.encode(&file.buf)?,
        })
    }


    async fn write_file(&self, mut file_data: FileData) -> crate::error::Result {
        file_data.buf = self.comp.decode(&file_data.buf)?;
        self.fs.write_file(file_data).await
    }

    async fn dir_entry_names<P: AsRef<Path> + Send + Sync>(
        &self,
        path: &P,
    ) -> crate::error::Result<Vec<PathBuf>> {
        self.fs.dir_entry_names(path).await
    }


    async fn create_dir<P: AsRef<Path> + Send>(&self, path: P) -> crate::error::Result {
        self.fs.create_dir(path).await
    }
}
