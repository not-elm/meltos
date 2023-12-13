use std::collections::HashMap;
use std::ops::Deref;
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


#[derive(Debug, Clone)]
pub struct ArcHashMap<K, V>(ArcMutex<HashMap<K, V>>)
where
    K: Send + Sync,
    V: Send + Sync;

impl<K, V> Deref for ArcHashMap<K, V>
where
    K: Send + Sync,
    V: Send + Sync,
{
    type Target = ArcMutex<HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<K, V> Default for ArcHashMap<K, V>
where
    K: Send + Sync,
    V: Send + Sync,
{
    fn default() -> ArcHashMap<K, V> {
        Self(ArcMutex::default())
    }
}
