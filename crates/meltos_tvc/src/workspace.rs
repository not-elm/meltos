use std::io;

use meltos_util::compression::CompressionBuf;
use meltos_util::compression::gz::Gz;

use crate::io::{FilePath, OpenIo, TvcIo};
use crate::object::{Object, ObjectHash};

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Open, Io>(pub(crate) TvcIo<Open, Io>)
    where
        Open: OpenIo<Io>,
        Io: io::Read + io::Write;


impl<Open, Io> WorkspaceIo<Open, Io>
    where
        Open: OpenIo<Io>,
        Io: io::Read + io::Write,
{
    pub fn convert_to_objs(&self, path: &str) -> std::io::Result<Vec<Object>> {
        let files = self.0.all_file_path(path)?;
        let mut objs = Vec::with_capacity(files.len());
        for path in files {
            objs.push(self.read_to_object(&path)?);
        }
        Ok(objs)
    }

    pub fn read_to_object(&self, path: &str) -> std::io::Result<Object> {
        let buf = self.0.try_read_to_end(path.as_ref())?;
        Ok(Object::new(FilePath::from_path(path), Gz.encode(&buf)?, ObjectHash(meltos_util::hash::hash(&buf))))
    }
}


impl<Open, Io> Default for WorkspaceIo<Open, Io>
    where
        Open: OpenIo<Io> + Default,
        Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvcIo::default())
    }
}


#[cfg(test)]
mod tests {
    use meltos_util::compression::CompressionBuf;
    use meltos_util::compression::gz::Gz;

    use crate::io::{OpenIo, TvcIo};
    use crate::io::mock::MockOpenIo;
    use crate::object::ObjectHash;
    use crate::workspace::WorkspaceIo;

    #[test]
    fn read_object() {
        let mock = MockOpenIo::default();
        let workspace = WorkspaceIo(TvcIo::new(mock.clone()));
        mock.write("hello/hello.txt", b"hello").unwrap();
        mock.write("hello/world", b"world").unwrap();
        let obj1 = workspace.read_to_object("hello/hello.txt").unwrap();
        assert_eq!(Gz.decode(&obj1.buf).unwrap(), b"hello");
        let obj2 = workspace.read_to_object("hello/world").unwrap();
        assert_eq!(Gz.decode(&obj2.buf).unwrap(), b"world");
    }

    #[test]
    fn read_all_objects_in_dir() {
        let mock = MockOpenIo::default();
        let workspace = WorkspaceIo(TvcIo::new(mock.clone()));
        mock.write("hello/hello.txt", b"hello").unwrap();
        mock.write("hello/world", b"world").unwrap();
        mock.write("hello/dir/main.sh", b"echo hi ").unwrap();
        let hashes = workspace.convert_to_objs("hello")
            .unwrap()
            .into_iter()
            .map(|obj|obj.hash)
            .collect::<Vec<ObjectHash>>();
        assert_eq!(hashes, vec![
            ObjectHash::new(b"hello"),
            ObjectHash::new(b"world"),
            ObjectHash::new(b"echo hi "),
        ])
    }
}