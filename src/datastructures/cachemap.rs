use core::cell::OnceCell;
use core::future::ready;
use std::hash::Hash;
use std::sync::Arc;

use dashmap::DashMap;
use futures::executor::block_on;
use tokio::task::spawn_blocking;

#[derive(Debug, Default)]
pub struct CacheMap<K, V>
where
    K: Eq + Hash,
{
    inner: DashMap<K, Arc<OnceCell<Arc<V>>>>,
}

impl<K, V> CacheMap<K, V>
where
    K: Eq + Hash,
    V: Send + Sync,
{
    /// Get or create a Cell for an entry. This sever the reference to &self, so the [`DashMap`] isn't locked
    pub fn get_entry_cell(&self, key: K) -> Arc<OnceCell<Arc<V>>> {
        self.inner
            .entry(key)
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone()
    }

    pub fn get_or_init(&self, key: K, v: impl FnOnce() -> V) -> Arc<V> {
        self.get_entry_cell(key)
            .get_or_init(|| (v)().into())
            .clone()
    }
}
