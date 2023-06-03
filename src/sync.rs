use std::any::type_name_of_val;
use std::borrow::{Borrow, Cow};
use std::collections::hash_map::{RandomState, RawEntryMut};
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::marker::Tuple;
use std::ops::Index;
use lock_api::Mutex;
use parking_lot::RawMutex;
use safe_once::sync::{OnceLock, RawFusedLock};
use safe_once_async::sync::AsyncOnceLock;
use crate::cow_entry::{CowEntry, CowEntryMut};
use crate::index_arena::IndexArena;
use crate::lazy_map::LazyMap;
use crate::stable_map::StableMap;

pub type OnceLockMap<K, V> = StableMap<K, OnceLock<V>, RandomState, RawFusedLock, RawMutex>;
pub type AsyncOnceLockMap<K, V> = StableMap<K, AsyncOnceLock<V>, RandomState, RawFusedLock, RawMutex>;
pub type LazyLockFn<K, V, F> = LazyMap<K, V, F, RandomState, RawFusedLock, RawMutex>;

#[test]
fn test_lazy() {
    let lazy: LazyMap<(usize, ), usize, _, _, _, _> = LazyLockFn::new(|x: usize| x + x);
    assert_eq!(4, *lazy.get((2, )));
    assert_eq!(4, lazy(2));
}

#[test]
fn test() {
    let map = OnceLockMap::<String, String>::default();
    assert_eq!("b", *map["a"].get_or_init(|| "b".to_string()));
    assert_eq!("b", *map["a"].get_or_init(|| "c".to_string()));
}