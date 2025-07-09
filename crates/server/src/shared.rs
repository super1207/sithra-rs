use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Weak},
    task::{Context, Poll},
};

use ahash::RandomState;
use futures_util::FutureExt;
use parking_lot::Mutex;
use tokio::sync::oneshot;

type OneshotMapInner<K, V> = Mutex<HashMap<K, Entry<V>, RandomState>>;

pub struct SharedOneshotMap<K, V>
where
    K: Eq + Hash + Send + Unpin + Clone + 'static,
{
    inner: Arc<OneshotMapInner<K, V>>,
}

impl<K, V> Clone for SharedOneshotMap<K, V>
where
    K: Eq + Hash + Send + Unpin + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K, V> Default for SharedOneshotMap<K, V>
where
    K: Eq + Hash + Send + Unpin + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SharedOneshotMap<K, V>
where
    K: Eq + Hash + Send + Unpin + Clone + 'static,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    pub fn register(&self, key: K) -> Option<ReceiverGuard<K, V>> {
        let (tx, rx) = oneshot::channel();
        let entry = Entry {
            tx,
            _marker: PhantomData,
        };
        {
            let mut map = self.inner.lock();
            map.insert(key.clone(), entry);
        }
        Some(ReceiverGuard {
            key,
            rx,
            map: Arc::downgrade(&self.inner),
        })
    }

    pub fn complete(&self, key: &K, value: V) -> Option<V> {
        let entry = {
            let mut map = self.inner.lock();
            if let Some(entry) = map.remove(key) {
                entry
            } else {
                return Some(value);
            }
        };
        entry.tx.send(value).err()
    }
}

pub struct Entry<V> {
    tx:      oneshot::Sender<V>,
    _marker: PhantomData<V>,
}

pub struct ReceiverGuard<K, V>
where
    K: Eq + Hash + Send + Unpin + 'static,
{
    key: K,
    rx:  oneshot::Receiver<V>,
    map: Weak<OneshotMapInner<K, V>>,
}

impl<K, V> Future for ReceiverGuard<K, V>
where
    K: Eq + Hash + Send + Unpin + 'static,
{
    type Output = Result<V, oneshot::error::RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.rx.poll_unpin(cx)
    }
}

impl<K, V> Drop for ReceiverGuard<K, V>
where
    K: Eq + Hash + Send + Unpin + 'static,
{
    fn drop(&mut self) {
        if let Some(map) = self.map.upgrade() {
            map.lock().remove(&self.key);
        }
    }
}
