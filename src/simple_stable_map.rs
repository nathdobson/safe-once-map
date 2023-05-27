use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use lock_api::{Mutex, RawMutex};
use safe_once::raw::RawOnce;
use crate::cow_entry::CowEntry;
use crate::index_arena::IndexArena;
use crate::stable_map::StableMapImpl;

pub struct SimpleStableMap<K, V, S, RO: RawOnce, RM> {
    arena: IndexArena<RO, V>,
    map: Mutex<RM, HashMap<K, usize, S>>,
}

impl<K, V, S, RO, RM> Default for SimpleStableMap<K, V, S, RO, RM>
    where
        K: Eq + Hash,
        V: Default,
        S: BuildHasher + Default,
        RO: RawOnce,
        RM: RawMutex,
{
    fn default() -> Self {
        Self {
            arena: IndexArena::new(1),
            map: Mutex::new(HashMap::with_hasher(default())),
        }
    }
}

impl<K, V, S, RO, RM> StableMapImpl for SimpleStableMap<K, V, S, RO, RM>
    where
        K: Eq + Hash,
        V: Default,
        S: BuildHasher,
        RO: RawOnce,
        RM: RawMutex {
    type Key = K;
    type Value = V;
    fn get_or_insert<Q>(&self, key: Cow<Q>) -> &Self::Value where Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>, Self::Key: Borrow<Q> {
        let ref mut map = *self.map.lock();
        let len = map.len();
        self.arena.get(0, *map.cow_entry_mut(key).or_insert_with(|| len))
    }
}
