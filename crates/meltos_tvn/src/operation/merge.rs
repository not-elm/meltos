use crate::file_system::{FileSystem, FsIo};

pub struct Merge<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read
{

    fs: FsIo<Fs, Io>
}




impl<Fs, Io> Merge<Fs, Io>
where
    Fs: FileSystem<Io>,
    Io: std::io::Write + std::io::Read
{
    pub fn execute(&self, config: MergeConfig){

    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MergeConfig{

}
