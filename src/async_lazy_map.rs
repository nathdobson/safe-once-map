use std::collections::hash_map::RandomState;
use std::future::Future;
use std::hash::{BuildHasher, Hash};
use std::marker::Tuple;
use parking_lot::RawMutex;
use safe_once::cell::RawOnceCell;
use safe_once::raw::RawOnce;
use safe_once::sync::RawOnceLock;
use safe_once_async::async_lazy::AsyncLazy;
use safe_once_async::async_once::AsyncOnce;
use safe_once_async::cell::AsyncRawOnceCell;
use safe_once_async::detached::{Detached, detached};
use safe_once_async::raw::AsyncRawOnce;
use safe_once_async::sync::AsyncRawOnceLock;
use crate::raw_cell_mutex::RawCellMutex;
use crate::stable_map::StableMap;

pub struct AsyncLazyMap<K, V, S, RO: RawOnce, ROA: AsyncRawOnce, RM> {
    callback: Box<dyn Fn(K) -> Detached<V>>,
    map: StableMap<K, AsyncOnce<ROA, V>, S, RO, RM>,
}

pub type AsyncLazyCellMap<K, V, S = RandomState> = AsyncLazyMap<K, V, S, RawOnceCell, AsyncRawOnceCell, RawCellMutex>;
pub type AsyncLazyLockMap<K, V, S = RandomState> = AsyncLazyMap<K, V, S, RawOnceLock, AsyncRawOnceLock, RawMutex>;

impl<
    K: Eq + Hash + Clone,
    V: 'static + Send,
    S: Default + BuildHasher,
    RO: RawOnce,
    ROA: AsyncRawOnce,
    RM: lock_api::RawMutex
> AsyncLazyMap<K, V, S, RO, ROA, RM> {
    pub fn new<F, Fu>(callback: F) -> Self where F: 'static + Fn(K) -> Fu, Fu: 'static + Send + Future<Output=V> {
        AsyncLazyMap {
            callback: Box::new(move |k| {
                detached(callback(k))
            }),
            map: StableMap::new(),
        }
    }
    pub async fn get(&self, key: K) -> &V {
        self.map[&key].get_or_init_detached(|| (self.callback)(key)).await
    }
}

#[tokio::test]
async fn test_async_lazy_map() {
    let map = AsyncLazyLockMap::<String, String>::new(|x| async { x });
    assert_eq!(map.get("a".to_string()).await, "a");
}