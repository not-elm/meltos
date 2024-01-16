

use meltos::room::RoomId;
use meltos_tvc::file_system::{FileSystem, Stat};
use meltos_tvc::file_system::std_fs::StdFileSystem;

use crate::path::room_resource_dir;

#[derive(Debug, Clone)]
pub struct BackendFileSystem<Fs = StdFileSystem> {
    room_id: RoomId,
    fs: Fs,
}

impl<Fs> BackendFileSystem<Fs> {
    #[inline]
    pub const fn new(room_id: RoomId, fs: Fs) -> BackendFileSystem<Fs> {
        Self {
            room_id,
            fs,
        }
    }


    #[inline(always)]
    fn trim(&self, path: String) -> String {
        path.trim_start_matches(&self.as_path(&path)).to_string()
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        room_resource_dir(&self.room_id).join(path).to_str().unwrap().to_string()
    }
}

impl<Fs: FileSystem> FileSystem for BackendFileSystem<Fs> {
    #[inline(always)]
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        self.fs.stat(&self.as_path(path))
    }

    #[inline(always)]
    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        self.fs.write_file(&self.as_path(path), buf)
    }

    #[inline(always)]
    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        self.fs.create_dir(&self.as_path(path))
    }

    #[inline(always)]
    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        self.fs.read_file(&self.as_path(path))
    }

    #[inline(always)]
    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        Ok(self.fs
            .read_dir(&self.as_path(path))?
            .map(|files| files.into_iter().map(|path| self.trim(path)).collect()))
    }

    #[inline(always)]
    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let files = self
            .fs
            .all_files_in(&self.as_path(path))?;
        Ok(files
            .into_iter()
            .map(|file| self.trim(file))
            .collect())
    }

    #[inline(always)]
    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(&self.as_path(path))
    }
}


