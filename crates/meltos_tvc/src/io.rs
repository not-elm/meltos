mod file;
mod mock;

use std::path::Path;

pub trait OpenIo<Io: io::Read + io::Write> {
    fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Io>;

    
    fn read_to_end<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.open(path)?.read_to_end(&mut buf)?;
        Ok(buf)
    }

    
    fn write<P: AsRef<Path>>(&self, path: P, buf: &[u8]) -> io::Result<()> {
        self.open(path)?.write_all(buf)
    }
}
