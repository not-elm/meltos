use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

#[derive(Default)]
pub struct SharedMutex<S>(Arc<Mutex<S>>);


impl<S> SharedMutex<S> {
    pub async fn lock(&self) -> MutexGuard<S> {
        self.0.lock().await
    }
}


impl<S> Clone for SharedMutex<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
