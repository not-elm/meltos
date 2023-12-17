use crate::error;
use crate::file_system::FileSystem;

#[derive(Debug, Clone)]
pub struct Pull<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{

}


impl<Fs, Io> Pull<Fs, Io>
    where
        Fs: FileSystem<Io>,
        Io: std::io::Write + std::io::Read,
{
    pub fn pull(&self) -> error::Result{
        todo!()
    }
}


