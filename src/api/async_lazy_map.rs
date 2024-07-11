use crate::sync::AsyncLazyLockMap;
use crate::util::StableMap;
use parking_lot::RawMutex;
use safe_once::api::raw::RawFused;
use safe_once::cell::RawFusedCell;
use safe_once::sync::RawFusedLock;
use safe_once_async::async_lazy::AsyncLazy;
use safe_once_async::async_once::AsyncOnce;
use safe_once_async::cell::AsyncRawFusedCell;
use safe_once_async::detached::DetachedFuture;
use safe_once_async::raw::AsyncRawFused;
use safe_once_async::sync::AsyncRawFusedLock;
use std::collections::hash_map::RandomState;
use std::future::Future;
use std::hash::{BuildHasher, Hash};
use std::marker::Tuple;
use std::pin::Pin;
use tokio::task::JoinHandle;

pub struct AsyncLazyMap<K, F, Fu: DetachedFuture, S, RF: RawFused, ARF: AsyncRawFused, RM> {
    callback: F,
    map: StableMap<K, AsyncOnce<ARF, Fu>, S, RF, RM>,
}

impl<
        K: Eq + Hash + Clone,
        F,
        Fu,
        S: Default + BuildHasher,
        RO: RawFused,
        ARF: AsyncRawFused,
        RM: lock_api::RawMutex,
    > AsyncLazyMap<K, F, Fu, S, RO, ARF, RM>
where
    F: 'static + Fn(K) -> Fu,
    Fu: 'static + Unpin + DetachedFuture<Output: 'static + Send>,
{
    pub fn new(callback: F) -> Self {
        AsyncLazyMap {
            callback,
            map: StableMap::new(),
        }
    }
    pub async fn get(&self, key: K) -> &Fu::Output {
        self.map[&key]
            .get_or_init_detached(|| (self.callback)(key))
            .await
    }
}

#[cfg(test)]
mod test {
    use crate::sync::AsyncLazyLockMap;
    use safe_once_async::detached::{spawn_transparent, DetachedFuture, JoinTransparent};
    use std::pin::Pin;

    #[tokio::test]
    async fn test_async_lazy_map() {
        use futures::FutureExt;
        struct Foo;
        let map =
            AsyncLazyLockMap::<String, JoinTransparent<String>>::new(Box::new(|x: String| {
                spawn_transparent(async { x })
            }));
        assert_eq!(map.get("a".to_string()).await, "a");
    }
}
