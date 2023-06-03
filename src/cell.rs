use std::borrow::{Borrow, Cow};
use std::cell::{Ref, RefCell};
use std::collections::hash_map::{RandomState, RawEntryMut, RawVacantEntryMut};
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::mem;
use std::ops::Index;
use safe_once::cell::{OnceCell, RawFusedCell};
use safe_once_async::cell::{AsyncOnceCell, AsyncRawFusedCell};
use crate::api::async_lazy_map::AsyncLazyMap;
use crate::api::lazy_map::LazyMap;
use crate::util::{RawCellMutex, StableMap};

pub type OnceCellMap<K, V> = StableMap<K, OnceCell<V>, RandomState, RawFusedCell, RawCellMutex>;
pub type AsyncOnceCellMap<K, V> = StableMap<K, AsyncOnceCell<V>, RandomState, RawFusedCell, RawCellMutex>;
pub type LazyCellMap<K, V, F> = LazyMap<K, V, F, RandomState, RawFusedCell, RawCellMutex>;
pub type AsyncLazyCellMap<K, V, S = RandomState> = AsyncLazyMap<K, V, S, RawFusedCell, AsyncRawFusedCell, RawCellMutex>;


#[test]
fn test_lazy() {
    // use crate::lazy_map::LazyFn;

    let lazy: LazyMap<(usize, ), usize, _, _, _, _> = LazyCellMap::new(|x: usize| x + x);
    assert_eq!(4, *lazy.get((2, )));
    assert_eq!(4, lazy(2));
}

#[test]
fn test() {
    let map = OnceCellMap::<String, String>::default();
    assert_eq!("b", *map["a"].get_or_init(|| "b".to_string()));
    assert_eq!("b", *map["a"].get_or_init(|| "c".to_string()));
}