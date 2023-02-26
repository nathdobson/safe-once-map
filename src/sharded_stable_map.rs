use std::borrow::{Borrow, Cow};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::ops::Index;
use lock_api::{Mutex, RawMutex};
use safe_once::raw::RawOnce;
use safe_once::sync::RawOnceLock;
use crate::index_arena::IndexArena;
use parking_lot::RawMutex as ParkingLotRawMutex;
use crate::cow_entry::CowEntry;
use crate::stable_map::StableMap;

pub struct ShardedStableMap<K, V, S, RO: RawOnce, RM> {
    arena: IndexArena<RO, V>,
    hasher: S,
    shards: Vec<Mutex<RM, HashMap<K, usize, S>>>,
}

impl<K, V, S, RO: RawOnce, RM: RawMutex> Default for ShardedStableMap<K, V, S, RO, RM> where S: BuildHasher + Default, K: Hash + Eq, V: Default {
    fn default() -> Self {
        Self::with_shards(128)
    }
}

impl<K, V, S, RO: RawOnce, RM: RawMutex> ShardedStableMap<K, V, S, RO, RM> where S: BuildHasher, K: Hash + Eq, V: Default {
    pub fn with_shards(shards: usize) -> Self where S: Default {
        assert!(shards.is_power_of_two());
        Self {
            arena: IndexArena::new(shards),
            hasher: default(),
            shards: (0..shards).map(|_| Mutex::new(HashMap::with_hasher(default()))).collect(),
        }
    }
}

impl<K, V, S, RO: RawOnce, RM: RawMutex> StableMap for ShardedStableMap<K, V, S, RO, RM> where S: BuildHasher, K: Hash + Eq, V: Default {
    type Key = K;
    type Value = V;
    fn get_or_insert<Q>(&self, key: Cow<Q>) -> &Self::Value where Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>, Self::Key: Borrow<Q> {
        let shard = (self.hasher.hash_one(&key) as usize) & (self.shards.len() - 1);
        let ref mut map = *self.shards[shard].lock();
        let len = map.len();
        self.arena.get(shard, *map.cow_entry_mut(key).or_insert_with(|| len))
    }
}