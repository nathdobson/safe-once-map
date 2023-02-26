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
use safe_once::raw::RawOnce;
use safe_once::sync::{OnceLock, RawOnceLock};
use crate::cow_entry::{CowEntry, CowEntryMut};
use crate::index_arena::IndexArena;
use crate::lazy_map::LazyFn;
use crate::once_map::OnceMap;
use crate::sharded_stable_map::ShardedStableMap;
use crate::simple_stable_map::SimpleStableMap;
use crate::stable_map::StableMap;

pub type RawOnceLockMap<K: Eq + Hash, V> = impl Default + StableMap<Key=K, Value=OnceLock<V>>;

fn force_impl<K: Eq + Hash, V>(map: ShardedStableMap<K, OnceLock<V>, RandomState, RawOnceLock, RawMutex>) -> RawOnceLockMap<K, V> {
    map
}

pub type OnceLockMap<K, V> = OnceMap<K, V, RawOnceLockMap<K, V>>;

pub type LazyLockFn<K, V, F = fn(K) -> V> = LazyFn<K, V, F, RawOnceLockMap<K, V>>;

#[test]
fn test_lazy() {
    let lazy: LazyFn<(usize, ), usize, _, _> = LazyLockFn::new(|x: usize| x + x);
    assert_eq!(4, *lazy.get((2, )));
    assert_eq!(4, lazy(2));
}

#[test]
fn test() {
    let map = OnceLockMap::<String, String>::new();
    assert_eq!("b", *map["a"].get_or_init(|| "b".to_string()));
    assert_eq!("b", *map["a"].get_or_init(|| "c".to_string()));
}