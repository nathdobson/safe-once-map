use std::borrow::{Borrow, Cow};
use std::cell::{Ref, RefCell};
use std::collections::hash_map::{RandomState, RawEntryMut, RawVacantEntryMut};
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::mem;
use std::ops::Index;
use safe_once::cell::{OnceCell, RawOnceCell};
use crate::cow_entry::{CowEntry, CowEntryMut};
use crate::index_arena::IndexArena;
use crate::lazy_map::LazyFn;
use crate::raw_cell_mutex::RawCellMutex;
use crate::stable_map::{StableMap, StableMapImpl};
use crate::simple_stable_map::SimpleStableMap;

pub type RawOnceCellMap<K: Eq + Hash, V> = SimpleStableMap<K, V, RandomState, RawOnceCell, RawCellMutex>;

pub type OnceCellMap<K, V> = StableMap<RawOnceCellMap<K, V>>;

pub type LazyCellFn<K, V, F> = LazyFn<K, V, F, RawOnceCellMap<K, OnceCell<V>>>;

#[test]
fn test_lazy() {
    // use crate::lazy_map::LazyFn;

    let lazy: LazyFn<(usize, ), usize, _, _> = LazyCellFn::new(|x: usize| x + x);
    assert_eq!(4, *lazy.get((2, )));
    assert_eq!(4, lazy(2));
}

#[test]
fn test() {
    let map = OnceCellMap::<String, OnceCell<String>>::default();
    assert_eq!("b", *map["a"].get_or_init(|| "b".to_string()));
    assert_eq!("b", *map["a"].get_or_init(|| "c".to_string()));
}