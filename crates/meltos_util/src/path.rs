use std::path::{Path, PathBuf};

pub trait AsUri {
    fn as_uri(&self) -> String;
}

impl AsUri for Path {
    #[inline(always)]
    fn as_uri(&self) -> String {
        self.to_str().unwrap().replace('\\', "/")
    }
}

impl AsUri for PathBuf {
    #[inline(always)]
    fn as_uri(&self) -> String {
        self.to_str().unwrap().replace('\\', "/")
    }
}