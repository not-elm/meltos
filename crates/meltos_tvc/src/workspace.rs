use std::io;
use std::path::Path;

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
    pub fn read_objects_from_dir(&self, path: impl AsRef<Path>) -> std::io::Result<Vec<Object>> {
        let mut objects = Vec::new();
        self.scan_objects(&mut objects, path)?;
        Ok(objects)
    }

    pub fn read_to_object(&self, path: impl AsRef<Path>) -> std::io::Result<Object> {
        let buf = self.0.try_read_to_end(path.as_ref())?;
        Ok(Object::new(FilePath::from_path(path), Gz.encode(&buf)?, ObjectHash(meltos_util::hash::hash(&buf))))
    }


    fn scan_objects(&self, objects: &mut Vec<Object>, path: impl AsRef<Path>) -> std::io::Result<()> {
        let p: &Path = path.as_ref();

        if p.is_dir() {
            for entry in p.read_dir()? {
                self.scan_objects(objects, entry?.path())?;
            }
        } else {
            objects.push(self.read_to_object(path)?);
        }

        Ok(())
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
}