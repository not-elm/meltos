use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::io::OpenIo;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct MockOpenIo(Arc<Mutex<HashMap<String, MockIo>>>);


impl OpenIo<MockIo> for MockOpenIo {
    fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<MockIo> {
        let io = MockIo::default();
        self.0.lock().unwrap().insert(path.as_ref().to_str().unwrap().to_string(), MockIo(Arc::clone(&io.0)));
        Ok(io)
    }
}


#[derive(Default)]
pub struct MockIo(Arc<Mutex<Vec<u8>>>);


impl Read for MockIo {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.write(&self.0.lock().unwrap())
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

