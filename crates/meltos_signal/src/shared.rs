use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

#[derive(Default, Debug)]
pub struct SharedMutex<S>(Arc<Mutex<S>>);


impl<S> SharedMutex<S> {
    pub async fn lock(&self) -> MutexGuard<S> {
        self.0.lock().await
    }
}


impl<S> From<S> for SharedMutex<S> {
    fn from(value: S) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}


impl<S> Clone for SharedMutex<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
