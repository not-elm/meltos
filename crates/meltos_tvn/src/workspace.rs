use std::io;

use meltos_util::compression::gz::Gz;
use meltos_util::compression::CompressionBuf;

use crate::io::{FilePath, OpenIo, TvnIo};
use crate::object::{Object, ObjectHash};

#[derive(Debug, Clone)]
pub struct WorkspaceIo<Open, Io>(pub(crate) TvnIo<Open, Io>)
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write;


impl<Open, Io> WorkspaceIo<Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    pub fn convert_to_objs(&self, path: &str) -> std::io::Result<ObjectIter<Open, Io>> {
        let files = self.0.all_file_path(path)?;
        Ok(ObjectIter {
            files,
            index: 0,
            io: &self.0,
        })
    }
}


impl<Open, Io> Default for WorkspaceIo<Open, Io>
where
    Open: OpenIo<Io> + Default,
    Io: io::Read + io::Write,
{
    fn default() -> Self {
        Self(TvnIo::default())
    }
}


pub struct ObjectIter<'a, Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    files: Vec<String>,
    index: usize,
    io: &'a TvnIo<Open, Io>,
}


impl<'a, Open, Io> Iterator for ObjectIter<'a, Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    type Item = std::io::Result<Object>;

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


impl<'a, Open, Io> ObjectIter<'a, Open, Io>
where
    Open: OpenIo<Io>,
    Io: io::Read + io::Write,
{
    fn read_to_obj(&self) -> std::io::Result<Object> {
        let path = self.files.get(self.index).unwrap();
        let buf = self.io.try_read_to_end(path.as_ref())?;
        Ok(Object::new(
            FilePath::from_path(path),
            Gz.encode(&buf)?,
            ObjectHash(meltos_util::hash::hash(&buf)),
        ))
    }
}


#[cfg(test)]
mod tests {
    use crate::io::mock::MockOpenIo;
    use crate::io::{OpenIo, TvnIo};
    use crate::object::ObjectHash;
    use crate::workspace::WorkspaceIo;

    #[test]
    fn read_all_objects_in_dir() {
        let mock = MockOpenIo::default();
        let workspace = WorkspaceIo(TvnIo::new(mock.clone()));
        mock.write("hello/hello.txt", b"hello").unwrap();
        mock.write("hello/world", b"world").unwrap();
        mock.write("hello/dir/main.sh", b"echo hi ").unwrap();
        let mut hashes = workspace
            .convert_to_objs("hello")
            .unwrap()
            .into_iter()
            .map(|obj| obj.unwrap().hash)
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
