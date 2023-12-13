use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::io::OpenIo;

#[derive(Debug, Clone, Default)]
pub struct MockOpenIo(Arc<Mutex<HashMap<String, MockIo>>>);


impl OpenIo<MockIo> for MockOpenIo {
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Option<MockIo>> {
        let map = self.0.lock().unwrap();
        let key = path.as_ref().to_str().unwrap().to_string();
        Ok(map.get(&key).cloned())
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<MockIo> {
        let mut map = self.0.lock().unwrap();
        let key = path.as_ref().to_str().unwrap().to_string();
        if !map.contains_key(&key) {
            let io = MockIo::default();
            map.insert(key.clone(), io);
        }
        Ok(MockIo(Arc::clone(&map.get(&key).unwrap().0)))
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
