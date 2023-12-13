use std::fs::File;
use std::path::Path;
use crate::io::OpenIo;


#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FileOpen;



impl OpenIo<File> for FileOpen{
    #[inline(always)]
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<File> {
        let path: &Path = path.as_ref();
        if path.is_dir(){
            return Err(std::io::Error::other("path type should be file"));
        }
        if let Some(parent) = path.parent(){
            std::fs::create_dir_all(parent)?;
        }
        
        File::open(path)
    }
}