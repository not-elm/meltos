use std::path::Path;

pub fn delete_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path: &Path = path.as_ref();
    if path.exists() && path.is_dir() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
