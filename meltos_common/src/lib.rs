pub mod error;
pub mod compression;
pub mod fs;


#[cfg(test)]
pub(crate) mod test_util {
    use std::fs;
    use std::future::Future;

    use futures::FutureExt;

    use crate::error::MelResult;

    pub fn create_tests_dir() -> std::io::Result<()> {
        let _ = fs::create_dir("tests");
        let _ = fs::write("./tests/hello.txt", "hello");
        Ok(())
    }


    pub async fn unwind(
        f: impl Future<Output=MelResult>,
    ) -> MelResult {
        delete_dir();
        let result = std::panic::AssertUnwindSafe(f).catch_unwind().await;
        delete_dir();
        result.unwrap().unwrap();
        Ok(())
    }

    pub fn delete_dir() {
        let _ = fs::remove_dir_all("./tests");
    }
}