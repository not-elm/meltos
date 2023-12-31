use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use std::sync::{Arc, Mutex};

use crate::file_system::FileSystem;

#[derive(Clone, Default)]
pub struct MockFileSystem(pub Arc<Mutex<HashMap<String, Vec<u8>>>>);

impl MockFileSystem {
    #[cfg(test)]
    pub fn force_write(&self, path: &str, buf: &[u8]) {
        self.write(path, buf).unwrap();
    }
}



fn insert_suffix(path: &str) -> String{
    match path {
        "." | "./" => path.to_string(),
        _ =>   {
            if path.starts_with("./"){
                path.to_string()
            }else{
                format!("./{}", path.trim_start_matches("/"))
            }
        }
    }

}

impl FileSystem for MockFileSystem {
    fn write(&self, path: &str, buf: &[u8]) -> std::io::Result<()> {
        let mut map = self.0.lock().unwrap();
        map.insert(insert_suffix(path), buf.to_vec());
        Ok(())
    }

    fn read(&self, path: &str) -> std::io::Result<Option<Vec<u8>>> {
        let map = self.0.lock().unwrap();
        Ok(map.get(&insert_suffix(path)).cloned())
    }

    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        let map = self.0.lock().unwrap();
        let path = insert_suffix(path);
        Ok(map
            .keys()
            .filter(|key| key.starts_with(&path))
            .cloned()
            .collect())
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.0.lock().unwrap().remove(path);
        Ok(())
    }
}

impl Debug for MockFileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (key, _) in self.0.lock().unwrap().iter_mut() {
            f.write_fmt(format_args!("{key}\n"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::file_system::mock::MockFileSystem;
    use crate::file_system::FileSystem;

    #[test]
    fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let mock = MockFileSystem::default();

        mock.write("buf1", &buf1).unwrap();
        mock.write("buf2", &buf2).unwrap();
        assert_eq!(mock.read("buf1").unwrap().unwrap(), buf1.to_vec());
        assert_eq!(mock.read("buf2").unwrap().unwrap(), buf2.to_vec());
    }
}
