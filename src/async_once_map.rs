use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use parking_lot::RawMutex;
use safe_once::cell::RawOnceCell;
use safe_once::raw::RawOnce;
use safe_once::sync::RawOnceLock;
use safe_once_async::async_once::AsyncOnce;
use safe_once_async::cell::AsyncRawOnceCell;
use safe_once_async::raw::AsyncRawOnce;
use safe_once_async::sync::AsyncRawOnceLock;
use crate::raw_cell_mutex::RawCellMutex;
use crate::stable_map::StableMap;

// pub struct AsyncOnceMap<K, V, S, RO: RawOnce, ROA: AsyncRawOnce, RM> {
//     map: StableMap<K, AsyncOnce<ROA, V>, S, RO, RM>,
// }

pub type AsyncOnceMap<K, V, S, RO, ROA, RM> = StableMap<K, AsyncOnce<ROA, V>, S, RO, RM>;
pub type AsyncOnceCellMap<K, V, S = RandomState> = AsyncOnceMap<K, V, S, RawOnceCell, AsyncRawOnceCell, RawCellMutex>;
pub type AsyncOnceLockMap<K, V, S = RandomState> = AsyncOnceMap<K, V, S, RawOnceLock, AsyncRawOnceLock, RawMutex>;
//
// impl<
//     K: Eq + Hash + Clone,
//     V: 'static + Send,
//     S: Default + BuildHasher,
//     RO: RawOnce,
//     ROA: AsyncRawOnce,
//     RM: lock_api::RawMutex
// > AsyncOnceMap<K, V, S, RO, ROA, RM> {
//     pub fn new() -> Self{
//         AsyncOnceMap {
//             map: StableMap::new(),
//         }
//     }
// }

#[tokio::test]
async fn test_async_lazy_map() {
    let map = AsyncOnceLockMap::<String, String>::new();
    assert_eq!(map["a"].get_or_init(|| async { "b".to_string() }).await, "b");
}