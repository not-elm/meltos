use std::io;

use crate::file_system::{FilePath, FileSystem, FsIo};
use crate::object::file::FileObj;


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
    pub fn new(fs: Fs) -> WorkspaceIo<Fs, Io> {
        Self(FsIo::new(fs))
    }


    pub fn convert_to_objs(&self, path: &str) -> std::io::Result<ObjectIter<Fs, Io>> {
        let files = self
            .0
            .all_file_path(path)?
            .into_iter()
            .filter(|path| !path.starts_with("./.meltos") && !path.starts_with(".meltos"))
            .collect();
        Ok(ObjectIter {
            files,
            index: 0,
            io: &self.0,
        })
    }

    pub fn unpack(&self, file_path: &FilePath, obj_buf: &[u8]) -> std::io::Result<()> {
        self.0.write(file_path, obj_buf)
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
    type Item = std::io::Result<(FilePath, FileObj)>;

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
    fn read_to_obj(&self) -> std::io::Result<(FilePath, FileObj)> {
        let path = self.files.get(self.index).unwrap();
        let buf = self.io.try_read(path.as_ref())?;
        Ok((FilePath::from_path(path), FileObj(buf)))
    }
}

#[cfg(test)]
mod tests {

    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::{FileSystem, FsIo};
    use crate::io::atomic::workspace::WorkspaceIo;
    use crate::object::{AsMeta, ObjHash};

    #[test]
    fn read_all_objects_in_dir() {
        let mock = MockFileSystem::default();
        let workspace = WorkspaceIo(FsIo::new(mock.clone()));
        mock.write("hello/hello.txt", b"hello").unwrap();
        mock.write("hello/world", b"world").unwrap();
        mock.write("hello/dir/main.sh", b"echo hi ").unwrap();
        let mut hashes = workspace
            .convert_to_objs("hello")
            .unwrap()
            .map(|obj| obj.unwrap().1.clone().as_meta().unwrap().hash)
            .collect::<Vec<ObjHash>>();
        hashes.sort();
        let mut expect = vec![
            ObjHash::new(b"FILE\0hello"),
            ObjHash::new(b"FILE\0world"),
            ObjHash::new(b"FILE\0echo hi "),
        ];
        expect.sort();
        assert_eq!(hashes, expect);
    }
}
