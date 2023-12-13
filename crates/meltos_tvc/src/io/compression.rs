use std::io::{Read, Write};
use std::path::Path;

use meltos_util::compression::CompressionBuf;

use crate::io::{OpenIo, TvcIo};

#[derive(Debug)]
pub struct CompressionOpen<Open, Io, Compression>(TvcIo<Open, Io>, Compression)
    where
        Open: OpenIo<Io>,
        Io: std::io::Read + std::io::Write,
        Compression: CompressionBuf;

impl<Open, Io, Compression> OpenIo<CompressionIo<Io, Compression>> for CompressionOpen<Open, Io, Compression>
    where
        Open: OpenIo<Io>,
        Io: std::io::Read + std::io::Write,
        Compression: CompressionBuf + Clone
{
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Option<CompressionIo<Io, Compression>>> {
        let io = self.0.open(path)?;
        Ok(io.map(|io| CompressionIo(io, self.1.clone())))
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<CompressionIo<Io, Compression>> {
        let io = self.0.create(path)?;
        Ok(CompressionIo(io, self.1.clone()))
    }
}


impl<Open, Io, Compression> Clone for CompressionOpen<Open, Io, Compression>
    where
        Open: OpenIo<Io> + Clone,
        Io: std::io::Read + std::io::Write,
        Compression: CompressionBuf + Clone
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}


impl<Open, Io, Compression> Default for CompressionOpen<Open, Io, Compression>
    where
        Open: OpenIo<Io> + Default,
        Io: std::io::Read + std::io::Write,
        Compression: CompressionBuf + Default
{
    fn default() -> Self {
        Self(TvcIo::default(), Compression::default())
    }
}

#[derive(Debug, Clone)]
pub struct CompressionIo<Io, Compression>(Io, Compression);


impl<Io, Compression> Read for CompressionIo<Io, Compression>
    where Io: Read,
          Compression: CompressionBuf
{
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(&mut self.1.decode(buf)?)
    }
}


impl<Io, Compression> Write for CompressionIo<Io, Compression>
    where Io: Write,
          Compression: CompressionBuf
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(&self.1.encode(buf)?)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(&self.1.encode(buf)?)
    }
}