use std::fs;
use std::future::Future;
use std::path::Path;

use futures::FutureExt;

pub fn create_tests_dir() {
    let _ = fs::create_dir("tests");
}


pub async fn unwind<P: AsRef<Path> + Send + Sync>(
    path: P,
    f: impl Future<Output = crate::error::Result>,
) -> crate::error::Result {
    create_tests_dir();
    let result = std::panic::AssertUnwindSafe(f).catch_unwind().await;
    delete_path(path);
    result.unwrap().unwrap();
    Ok(())
}

pub fn delete_path<P: AsRef<Path> + Send + Sync>(path: P) {
    if path.as_ref().is_dir() {
        let _ = fs::remove_dir_all("./tests");
    } else {
        let _ = fs::remove_file(path);
    }
}
