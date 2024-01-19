use crate::encode::Decodable;
use crate::error;
use crate::file_system::{FilePath, FileSystem};
use crate::object::{AsMeta, Obj, ObjHash};
use crate::object::file::FileObj;
use crate::object::tree::TreeObj;

pub struct ChangeFileMeta {
    pub path: FilePath,
    pub change: ChangeFile,
}

pub enum ChangeFile {
    Create(FileObj),
    Update(FileObj),
    Delete,
}

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Fs>
    where
        Fs: FileSystem,
{
    fs: Fs,
}

impl<Fs> WorkspaceIo<Fs>
    where
        Fs: FileSystem,
{
    #[inline]
    pub const fn new(fs: Fs) -> WorkspaceIo<Fs> {
        Self {
            fs,
        }
    }

    pub fn convert_to_objs(&self, path: &str) -> error::Result<ObjectIter<Fs>> {
        let files = self.files(path)?;

        Ok(ObjectIter {
            files,
            index: 0,
            io: &self.fs,
        })
    }

    #[inline(always)]
    pub fn files(&self, path: &str) -> error::Result<Vec<String>> {
        let path = match path {
            "." | "./" => "workspace".to_string(),
            path => format!("workspace/{}", path.trim_start_matches("workspace/")),
        };
        Ok(self.fs.all_files_in(&path)?)
    }

    pub fn changed_files(&self, mut trace_tree: TreeObj) -> error::Result<Vec<ChangeFileMeta>> {
        let mut changed_files = Vec::new();
        self.compare_trace(&mut trace_tree, &mut changed_files)?;
        for (path, _) in trace_tree.0.into_iter() {
            changed_files.push(ChangeFileMeta {
                path,
                change: ChangeFile::Delete,
            })
        }
        Ok(changed_files)
    }

    fn compare_trace(
        &self,
        trace_tree: &mut TreeObj,
        changed_files: &mut Vec<ChangeFileMeta>,
    ) -> error::Result {
        let files = self.files(".")?;
        for file_path in files {
            let path = FilePath(file_path);
            let file_obj = self.try_read(&path)?;
            if let Some(trace_obj_hash) = trace_tree.remove(&path) {
                self.diff(changed_files, path, file_obj, trace_obj_hash)?;
            } else {
                changed_files.push(ChangeFileMeta {
                    path,
                    change: ChangeFile::Create(file_obj),
                });
            }
        }
        Ok(())
    }

    fn diff(
        &self,
        changed_files: &mut Vec<ChangeFileMeta>,
        path: FilePath,
        file_obj: FileObj,
        trace_obj_hash: ObjHash,
    ) -> error::Result {
        let meta = file_obj.as_meta()?;
        if meta.hash == trace_obj_hash {
            Ok(())
        } else {
            changed_files.push(ChangeFileMeta {
                path,
                change: ChangeFile::Update(file_obj),
            });
            Ok(())
        }
    }

    pub fn try_read(&self, file_path: &FilePath) -> error::Result<FileObj> {
        match self.read(file_path)? {
            Some(file_obj) => Ok(file_obj),
            None => {
                Err(crate::error::Error::NotfoundWorkspaceFile(
                    file_path.clone(),
                ))
            }
        }
    }

    pub fn read(&self, file_path: &FilePath) -> error::Result<Option<FileObj>> {
        let Some(buf) = self.fs.read_file(&self.as_path(file_path))? else {
            return Ok(None);
        };
        Ok(Some(FileObj::decode(&buf)?))
    }

    pub fn unpack(&self, file_path: &FilePath, obj: &Obj) -> error::Result<()> {
        match obj {
            Obj::File(file) => {
                self.fs.write_file(file_path, &file.0)?;
                Ok(())
            }
            Obj::Delete(_) => {
                self.fs.delete(file_path)?;
                Ok(())
            }
            _ => Err(crate::error::Error::InvalidWorkspaceObj),
        }
    }

    #[inline(always)]
    fn as_path(&self, path: &str) -> String {
        format!("workspace/{path}")
    }
}

pub struct ObjectIter<'a, Fs>
    where
        Fs: FileSystem,
{
    files: Vec<String>,
    index: usize,
    io: &'a Fs,
}

impl<'a, Fs> Iterator for ObjectIter<'a, Fs>
    where
        Fs: FileSystem,
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

impl<'a, Fs> ObjectIter<'a, Fs>
    where
        Fs: FileSystem,
{
    fn read_to_obj(&self) -> std::io::Result<(FilePath, FileObj)> {
        let path = self.files.get(self.index).unwrap();
        let buf = self.io.try_read_file(path.as_ref())?;
        Ok((FilePath::from_path(path), FileObj(buf)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::file_system::{FilePath, FileSystem};
    use crate::file_system::mock::MockFileSystem;
    use crate::io::atomic::object::ObjIo;
    use crate::io::workspace::WorkspaceIo;
    use crate::object::{AsMeta, Obj, ObjHash};
    use crate::object::file::FileObj;

    #[test]
    fn read_all_objects_in_dir() {
        let mock = MockFileSystem::default();
        let workspace = WorkspaceIo::new(mock.clone());
        mock.write_file("workspace/hello/hello.txt", b"hello")
            .unwrap();
        mock.write_file("workspace/hello/world", b"world").unwrap();
        mock.write_file("workspace/hello/dir/main.sh", b"echo hi ")
            .unwrap();
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

    #[test]
    fn decode_buffer() {
        let mock = MockFileSystem::default();
        let workspace = WorkspaceIo::new(mock.clone());
        let obj = FileObj(b"hello".to_vec());
        let meta = obj.as_meta().unwrap();
        ObjIo::new(mock.clone())
            .write(&meta.hash, &meta.compressed_buf)
            .unwrap();
        workspace
            .unpack(&FilePath::from_path("hello.txt"), &Obj::File(obj))
            .unwrap();
        assert_eq!(mock.try_read_file("hello.txt").unwrap(), b"hello");
    }

    #[test]
    fn read_all_files() {
        let mock = MockFileSystem::default();

        let workspace = WorkspaceIo::new(mock.clone());
        mock.force_write("workspace/hello.txt", b"hello");
        mock.force_write("workspace/dist/index.js", b"index");
        let files = workspace.files(".").unwrap();
        assert_eq!(
            files.into_iter().collect::<HashSet<String>>(),
            vec![
                "workspace/hello.txt".to_string(),
                "workspace/dist/index.js".to_string(),
            ]
                .into_iter()
                .collect::<HashSet<String>>()
        );
    }
}
