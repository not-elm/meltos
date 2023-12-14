use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use crate::io::OpenIo;

#[derive(Debug, Clone, Default)]
pub struct MockOpenIo(pub Arc<Mutex<HashMap<String, MockIo>>>);


impl OpenIo<MockIo> for MockOpenIo {
    fn open_file(&self, path: &str) -> std::io::Result<Option<MockIo>> {
        let map = self.0.lock().unwrap();
        Ok(map.get(path).cloned())
    }


    fn all_file_path(&self, path: &str) -> std::io::Result<Vec<String>> {
        let map = self.0.lock().unwrap();
        Ok(map
            .keys()
            .filter(|key| key.starts_with(path))
            .cloned()
            .collect())
    }

    fn create(&self, path: &str) -> std::io::Result<MockIo> {
        let mut map = self.0.lock().unwrap();

        if !map.contains_key(path) {
            let io = MockIo::default();
            map.insert(path.to_string(), io);
        }
        Ok(MockIo(Arc::clone(&map.get(path).unwrap().0)))
    }

    fn delete(&self, path: &str) -> std::io::Result<()> {
        self.0.lock().unwrap().remove(path);
        Ok(())
    }
}


#[derive(Default, Debug, Clone)]
pub struct MockIo(Arc<Mutex<Vec<u8>>>);


impl Read for MockIo {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let b = self.0.lock().unwrap();
        buf[0..b.len()].copy_from_slice(b.as_slice());
        Ok(b.len())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        *buf = self.0.lock().unwrap().to_vec();
        Ok(buf.len())
    }
}

impl Write for MockIo {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }


    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        *self.0.lock().unwrap() = buf.to_vec();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::io::mock::MockOpenIo;
    use crate::io::OpenIo;

    #[test]
    fn read() {
        let buf1 = [0, 1, 2, 3];
        let buf2 = [5, 6, 7, 8];
        let io = MockOpenIo::default();

        io.write("buf1", &buf1).unwrap();
        io.write("buf2", &buf2).unwrap();
        assert_eq!(io.read_to_end("buf1").unwrap().unwrap(), buf1.to_vec());
        assert_eq!(io.read_to_end("buf2").unwrap().unwrap(), buf2.to_vec());
    }
}
