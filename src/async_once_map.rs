use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use parking_lot::RawMutex;
use safe_once::cell::RawFusedCell;
use safe_once::sync::RawFusedLock;
use safe_once_async::async_once::AsyncOnce;
use safe_once_async::cell::AsyncRawFusedCell;
use safe_once_async::sync::AsyncRawFusedLock;
use crate::raw_cell_mutex::RawCellMutex;
use crate::stable_map::StableMap;

pub type AsyncOnceMap<K, V, S, RO, ROA, RM> = StableMap<K, AsyncOnce<ROA, V>, S, RO, RM>;
pub type AsyncOnceCellMap<K, V, S = RandomState> = AsyncOnceMap<K, V, S, RawFusedCell, AsyncRawFusedCell, RawCellMutex>;
pub type AsyncOnceLockMap<K, V, S = RandomState> = AsyncOnceMap<K, V, S, RawFusedLock, AsyncRawFusedLock, RawMutex>;

#[tokio::test]
async fn test_async_lazy_map() {
    let map = AsyncOnceLockMap::<String, String>::new();
    assert_eq!(map["a"].get_or_init(async { "b".to_string() }).await, "b");
}