use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

use crate::file_system::mock::dir::MockDir;
use crate::file_system::mock::file::MockFile;
use crate::file_system::{FileSystem, Stat};

mod dir;
mod file;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MockEntry {
    File(MockFile),
    Dir(MockDir),
}

#[derive(Debug, Eq, PartialEq)]
pub enum MockEntryRef<'a> {
    File(&'a MockFile),
    Dir(&'a MockDir),
}

#[derive(Debug, Eq, PartialEq)]
pub enum MockEntryMut<'a> {
    File(&'a mut MockFile),
    Dir(&'a mut MockDir),
}

impl<'a> MockEntryMut<'a> {
    #[inline]
    pub fn stat(&self) -> Stat {
        match self {
            Self::File(file) => file.stat(),
            Self::Dir(dir) => dir.stat(),
        }
    }

    pub fn file(self) -> std::io::Result<&'a mut MockFile> {
        match self {
            Self::File(file) => Ok(file),
            Self::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn dir(self) -> std::io::Result<&'a mut MockDir> {
        match self {
            Self::Dir(dir) => Ok(dir),
            Self::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }
}

impl MockEntry {
    #[inline]
    pub fn stat(&self) -> Stat {
        match self {
            MockEntry::File(file) => file.stat(),
            MockEntry::Dir(dir) => dir.stat(),
        }
    }

    pub fn file(self) -> std::io::Result<MockFile> {
        match self {
            MockEntry::File(file) => Ok(file),
            MockEntry::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn file_mut(&mut self) -> std::io::Result<&mut MockFile> {
        match self {
            MockEntry::File(file) => Ok(file),
            MockEntry::Dir(_) => Err(std::io::Error::other("expect file bad was dir.")),
        }
    }

    pub fn dir_mut(&mut self) -> std::io::Result<&mut MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }

    pub fn dir_ref(&self) -> std::io::Result<&MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }

    pub fn dir(self) -> std::io::Result<MockDir> {
        match self {
            MockEntry::Dir(dir) => Ok(dir),
            MockEntry::File(_) => Err(std::io::Error::other("expect dir bad was file.")),
        }
    }
}

#[derive(Clone, Default)]
pub struct MockFileSystem(pub Arc<Mutex<MockDir>>);

impl MockFileSystem {
    #[cfg(test)]
    pub fn force_write(&self, path: &str, buf: &[u8]) {
        self.write_file(path, buf).unwrap();
    }
}

impl FileSystem for MockFileSystem {
    fn stat(&self, path: &str) -> std::io::Result<Option<Stat>> {
        let mut root = self.0.lock().unwrap();
        let Some(entry) = root.read(path) else {
            return Ok(None);
        };

        Ok(Some(entry.stat()))
    }

    fn write_file(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let mut root = self.0.lock().unwrap();
        root.write_file(path, buf);

        Ok(())
    }

    fn create_dir(&self, path: &str) -> std::io::Result<()> {
        let mut root = self.0.lock().unwrap();
        root.create_dir(path);

        Ok(())
    }

    fn read_file(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let mut root = self.0.lock().unwrap();
        let Some(entry) = root.read(path) else {
            return Ok(None);
        };
        Ok(Some(entry.file()?.buf.clone()))
    }

    fn read_dir(&self, path: &str) -> std::io::Result<Option<Vec<String>>> {
        let mut root = self.0.lock().unwrap();
        let Some(entry) = root.read(path) else {
            return Ok(None);
        };

        Ok(Some(entry.dir()?.entries.keys().cloned().collect()))
    }

    #[inline]
    fn all_files_in(&self, path: &str) -> std::io::Result<Vec<String>> {
        let mut root = self.0.lock().unwrap();
        let path = path.trim_start_matches("./").trim_end_matches('/');
        let Some(entry) = root.read(path) else {
            return Ok(Vec::with_capacity(0));
        };
        if let Ok(relative) = entry.dir() {
            let parent_path = if path == "." || path == "./" {
                None
            } else {
                Some(path.to_string())
            };
            Ok(relative.all_files(parent_path))
        } else {
            Ok(vec![path.to_string()])
        }
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        let mut root = self.0.lock().unwrap();
        if let Some(parent) = root.lookup_parent_dir(path) {
            parent.entries.remove(&entry_name(path));
        } else {
            root.entries.remove(path);
        }
        Ok(())
    }
}

impl Debug for MockFileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let root = self.0.lock().unwrap();
        f.write_fmt(format_args!("{root:?}"))
    }
}

fn parent_path(path: &str) -> Option<String> {
    let mut ps: Vec<&str> = path.split('/').collect();
    if ps.len() <= 1 {
        return None;
    }

    ps.pop();
    Some(ps.join("/"))
}

fn entry_name(path: &str) -> String {
    let mut ps: Vec<&str> = path.split('/').collect();
    ps.pop().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn read_root_dir() {
        let mock = MockFileSystem::default();
        let dir = mock.read_dir(".").unwrap();
        assert_eq!(dir.unwrap().len(), 0);
    }

    #[test]
    fn read_root_dir_with_files() {
        let mock = MockFileSystem::default();
        mock.force_write("1.txt", b"1");
        mock.force_write("2.txt", b"2");
        mock.force_write("3.txt", b"3");
        println!("ENTRY = {mock:?}");
        let dir = mock.read_dir(".").unwrap();
        assert_eq!(dir.unwrap().len(), 3);
    }

    #[test]
    fn create_src_dir() {
        let mock = MockFileSystem::default();
        mock.create_dir("src").unwrap();
        let dir = mock.try_read_dir("src").unwrap();
        assert_eq!(dir.len(), 0);

        mock.force_write("src/hello.txt", b"hello");
        let src = mock.try_read_dir("src").unwrap();
        assert_eq!(src.len(), 1);
    }

    #[test]
    fn create_parent_dirs() {
        let mock = MockFileSystem::default();
        mock.force_write("dist/hello.txt", b"hello");
        mock.force_write("dist/hello2.txt", b"hello");
        mock.force_write("dist/hello3.txt", b"hello");

        let dist = mock.try_read_dir("dist").unwrap();

        assert_eq!(dist.len(), 3);
    }

    #[test]
    fn read_hello_world() {
        let mock = MockFileSystem::default();
        mock.force_write("hello.txt", b"hello world");
        mock.force_write("dist/hello.txt", b"hello world");
        mock.force_write("dist/sample/hello.txt", b"hello world");

        let buf = mock.read_file("hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = mock.read_file("dist/hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = mock.read_file("dist/sample/hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
    }

    #[test]
    fn read_file_start_with_period() {
        let mock = MockFileSystem::default();
        mock.force_write("hello.txt", b"hello world");
        mock.force_write("dist/hello.txt", b"hello world");
        mock.force_write("dist/sample/hello.txt", b"hello world");

        let buf = mock.read_file("./hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = mock.read_file("./dist/hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
        let buf = mock.read_file("./dist/sample/hello.txt").unwrap();
        assert_eq!(buf, Some(b"hello world".to_vec()));
    }

    #[test]
    fn delete_file() {
        let mock = MockFileSystem::default();
        mock.force_write("hello.txt", b"hello world");
        mock.delete("hello.txt").unwrap();

        assert_eq!(mock.read_file("hello.txt").unwrap(), None);
    }

    #[test]
    fn delete_dir() {
        let mock = MockFileSystem::default();
        mock.create_dir("src").unwrap();
        mock.write_file("src/hello.txt", b"hello").unwrap();

        mock.force_write("dist/hello.txt", b"hello");
        mock.force_write("dist/sample/sample.js", b"console.log(`sample`)");

        assert_eq!(mock.read_dir("src").unwrap().unwrap().len(), 1);
        assert_eq!(mock.read_dir("dist/sample").unwrap().unwrap().len(), 1);

        mock.delete("src").unwrap();
        assert!(mock.read_dir("src").unwrap().is_none());
        assert_eq!(mock.read_dir("dist/sample").unwrap().unwrap().len(), 1);
        assert_eq!(mock.try_read_dir("dist").unwrap().len(), 2);

        mock.delete("dist/sample").unwrap();
        assert!(mock.read_dir("src").unwrap().is_none());
        assert!(mock.read_dir("dist/sample").unwrap().is_none());
        assert_eq!(mock.try_read_dir("dist").unwrap().len(), 1);
    }

    #[test]
    fn all_files_with_in_children() {
        let mock = MockFileSystem::default();
        mock.force_write("hello1.txt", b"hello");
        mock.force_write("hello2.txt", b"hello");
        mock.force_write("hello3.txt", b"hello");

        let mut files = mock.all_files_in(".").unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "hello1.txt".to_string(),
                "hello2.txt".to_string(),
                "hello3.txt".to_string(),
            ]
        );
    }

    #[test]
    fn all_files_recursive() {
        let mock = MockFileSystem::default();
        mock.force_write("hello1.txt", b"hello");
        mock.force_write("src/hello2.txt", b"hello");
        mock.force_write("src/dist/hello3.txt", b"hello");

        let mut files = mock.all_files_in(".").unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "hello1.txt".to_string(),
                "src/dist/hello3.txt".to_string(),
                "src/hello2.txt".to_string(),
            ]
        );
    }

    #[test]
    fn all_files_relative_to_src() {
        let mock = MockFileSystem::default();
        mock.force_write("hello1.txt", b"hello");
        mock.force_write("src/hello2.txt", b"hello");
        mock.force_write("src/dist/hello3.txt", b"hello");

        let mut files = mock.all_files_in("src").unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                "src/dist/hello3.txt".to_string(),
                "src/hello2.txt".to_string(),
            ]
        );
    }

    #[test]
    fn return_none_if_not_exists_entry() {
        let mock = MockFileSystem::default();
        mock.create_dir("src").unwrap();
        let stat = mock.stat("hello.txt").unwrap();
        assert_eq!(stat, None);
        let stat = mock.stat("src/hello.txt").unwrap();
        assert_eq!(stat, None);
    }

    #[test]
    fn stat_file() {
        let mock = MockFileSystem::default();
        mock.write_file("src/hello.txt", b"hello").unwrap();
        let stat = mock.stat("src/hello.txt").unwrap().unwrap();
        assert!(stat.is_file());
        assert_eq!(stat.size, b"hello".len() as u64);
    }

    #[test]
    fn stat_dir() {
        let mock = MockFileSystem::default();
        mock.create_dir("src").unwrap();
        let stat = mock.stat("src").unwrap().unwrap();
        assert!(stat.is_dir());
        assert_eq!(stat.size, 0);
    }

    #[test]
    fn update_dir_stat() {
        let mock = MockFileSystem::default();
        mock.create_dir("src").unwrap();

        mock.create_dir("src/dist").unwrap();
        let stat = mock.stat("src").unwrap().unwrap();
        assert_eq!(stat.size, 1);

        mock.write_file("src/hello.txt", b"hello world").unwrap();
        let stat = mock.stat("src").unwrap().unwrap();
        assert_eq!(stat.size, 2);
    }

    #[test]
    fn update_file_stat() {
        let mock = MockFileSystem::default();
        mock.write_file("src/hello.txt", b"hello world").unwrap();
        let stat1 = mock.stat("src/hello.txt").unwrap().unwrap();
        sleep(Duration::new(1, 100));
        mock.write_file("src/hello.txt", b"hello").unwrap();
        let stat2 = mock.stat("src/hello.txt").unwrap().unwrap();
        assert_eq!(stat1.create_time, stat2.create_time);
        assert_eq!(stat2.size, b"hello".len() as u64);

        assert!(stat1.update_time < stat2.update_time);
    }

    #[test]
    fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let mock = MockFileSystem::default();

        mock.write_file("buf1", &buf1).unwrap();
        mock.write_file("buf2", &buf2).unwrap();
        assert_eq!(mock.read_file("buf1").unwrap().unwrap(), buf1.to_vec());
        assert_eq!(mock.read_file("buf2").unwrap().unwrap(), buf2.to_vec());
    }
}