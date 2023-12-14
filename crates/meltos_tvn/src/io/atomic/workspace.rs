use std::io;

use crate::file_system::{FilePath, FileSystem, FsIo};
use crate::object::ObjectMeta;

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Fs, Io>(pub(crate) FsIo<Fs, Io>)
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write;

impl<Fs, Io> WorkspaceIo<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    #[inline]
    pub fn new(fs: Fs) -> WorkspaceIo<Fs, Io>{
        Self(FsIo::new(fs))
    }
    
    
    pub fn convert_to_objs(&self, path: &str) -> std::io::Result<ObjectIter<Fs, Io>> {
        let files = self.0.all_file_path(path)?;
        Ok(ObjectIter {
            files,
            index: 0,
            io: &self.0,
        })
    }

    pub fn unpack(&self, file_path: &FilePath, obj_buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(file_path, obj_buf)
    }
}

impl<Fs, Io> Default for WorkspaceIo<Fs, Io>
    where
        Fs: FileSystem<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(FsIo::default())
    }
}

pub struct ObjectIter<'a, Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    files: Vec<String>,
    index: usize,
    io: &'a FsIo<Fs, Io>,
}

impl<'a, Fs, Io> Iterator for ObjectIter<'a, Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    type Item = std::io::Result<ObjectMeta>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.files.len() {
            None
        } else {
            let obj = self.read_to_obj();
            self.index += 1;
            Some(obj)
        }
    }
}

impl<'a, Fs, Io> ObjectIter<'a, Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: io::Read + io::Write,
{
    fn read_to_obj(&self) -> std::io::Result<ObjectMeta> {
        let path = self.files.get(self.index).unwrap();
        let buf = self.io.try_read_to_end(path.as_ref())?;
        ObjectMeta::new(FilePath::from_path(path), buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::file_system::{FileSystem, FsIo};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::workspace::WorkspaceIo;
    use crate::object::ObjectHash;

    #[test]
    fn read_all_objects_in_dir() {
        let mock = MockFileSystem::default();
        let workspace = WorkspaceIo(FsIo::new(mock.clone()));
        mock.write_all("hello/hello.txt", b"hello").unwrap();
        mock.write_all("hello/world", b"world").unwrap();
        mock.write_all("hello/dir/main.sh", b"echo hi ").unwrap();
        let mut hashes = workspace
            .convert_to_objs("hello")
            .unwrap()
            .map(|obj| obj.unwrap().hash().clone())
            .collect::<Vec<ObjectHash>>();
        hashes.sort();
        let mut expect = vec![
            ObjectHash::new(b"hello"),
            ObjectHash::new(b"world"),
            ObjectHash::new(b"echo hi "),
        ];
        expect.sort();
        assert_eq!(hashes, expect);
    }
}
