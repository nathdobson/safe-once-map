use crate::util::cow_entry::CowEntry;
use crate::util::index_arena::IndexArena;
use lock_api::{Mutex, RawMutex};
use safe_once::api::raw::RawFused;
use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::ops::Index;

pub struct StableMap<K, V, S, RF: RawFused, RM> {
    arena: IndexArena<RF, V>,
    map: Mutex<RM, HashMap<K, usize, S>>,
}

impl<'q, K, V, S, RF: RawFused, RM, Q> Index<&'q Q> for StableMap<K, V, S, RF, RM>
where
    Q: ?Sized + Hash + Eq + ToOwned<Owned = K>,
    K: Borrow<Q> + Eq + Hash,
    S: Default + BuildHasher,
    V: Default,
    RM: RawMutex,
{
    type Output = V;
    fn index(&self, index: &'q Q) -> &Self::Output {
        self.get_or_insert(Cow::Borrowed(index))
    }
}

impl<K, V, S, RF, RM> Default for StableMap<K, V, S, RF, RM>
where
    K: Eq + Hash,
    V: Default,
    S: BuildHasher + Default,
    RF: RawFused,
    RM: RawMutex,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, S, RF, RM> StableMap<K, V, S, RF, RM>
where
    K: Eq + Hash,
    V: Default,
    S: Default + BuildHasher,
    RF: RawFused,
    RM: RawMutex,
{
    pub fn new() -> Self {
        Self {
            arena: IndexArena::new(),
            map: Mutex::new(HashMap::with_hasher(Default::default())),
        }
    }
    pub fn get_or_insert<'a, Q>(&'a self, key: Cow<Q>) -> &'a V
    where
        Q: ?Sized + Hash + Eq + ToOwned<Owned = K>,
        K: Borrow<Q>,
    {
        let ref mut map = *self.map.lock();
        let len = map.len();
        self.arena
            .get_or_init(*map.cow_entry_mut(key).or_insert_with(|| len))
    }
    pub fn len(&self) -> usize {
        self.map.lock().len()
    }
}
