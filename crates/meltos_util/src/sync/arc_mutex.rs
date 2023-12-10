use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

#[derive(Default, Debug)]
pub struct ArcMutex<S: Send + Sync>(Arc<Mutex<S>>);


impl<S: Send + Sync> ArcMutex<S> {
    pub async fn lock(&self) -> MutexGuard<S> {
        self.0.lock().await
    }
}


impl<S: Send + Sync> From<S> for ArcMutex<S> {
    fn from(value: S) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}


impl<S: Send + Sync> Clone for ArcMutex<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


unsafe impl<S: Send + Sync> Send for ArcMutex<S> {}


unsafe impl<S: Send + Sync> Sync for ArcMutex<S> {}
