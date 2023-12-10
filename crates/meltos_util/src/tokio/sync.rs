use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

#[derive(Default, Debug)]
pub struct ArcMutex<S>(Arc<Mutex<S>>);


impl<S> ArcMutex<S> {
    pub async fn lock(&self) -> MutexGuard<S> {
        self.0.lock().await
    }
}


impl<S> From<S> for ArcMutex<S> {
    fn from(value: S) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}


impl<S> Clone for ArcMutex<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
