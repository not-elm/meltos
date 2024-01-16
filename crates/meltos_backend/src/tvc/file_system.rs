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
        path
            .trim_start_matches(&format!("{}/", room_resource_dir(&self.room_id).to_str().unwrap()))
            .trim_start_matches("./")
            .to_string()
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        let dir = room_resource_dir(&self.room_id);
        let path = path.trim_start_matches(dir.to_str().unwrap());
        format!("{}/{}", dir.to_str().unwrap(), path.trim_start_matches('/'))
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
        Ok(self
            .fs
            .all_files_in(&self.as_path(path))?
            .into_iter()
            .map(|file| self.trim(file))
            .collect())
    }

    #[inline(always)]
    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.fs.delete(&self.as_path(path))
    }
}


#[cfg(test)]
mod tests {
    use meltos::room::RoomId;
    use meltos_tvc::file_system::FileSystem;
    use meltos_tvc::file_system::mock::MockFileSystem;

    use crate::tvc::file_system::BackendFileSystem;

    #[test]
    fn read_files_in_dir() {
        let mock = MockFileSystem::default();
        let fs = BackendFileSystem::new(RoomId::new(), mock.clone());
        fs.write_file("dir/hello.txt", b"hello").unwrap();
        fs.write_file("hello2.txt", b"hello").unwrap();
        let mut files = fs.all_files_in(".").unwrap();
        files.sort();
        assert_eq!(files, vec![
            "dir/hello.txt".to_string(),
            "hello2.txt".to_string(),
        ])
    }
}