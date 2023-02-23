use std::borrow::{Borrow, Cow};
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::ptr::NonNull;
use colosseum::unsync::Arena;
use parking_lot::Mutex;

struct UncheckedRef<T>(*const T);

impl<T> UncheckedRef<T> {
    fn new(x: &T) -> Self { UncheckedRef(x) }
    unsafe fn deref_unbounded<'a, 'b>(&'a self) -> &'b T { &*self.0 }
}

struct Shard<K, V, S> {
    arena: Arena<V>,
    map: HashMap<K, UncheckedRef<V>, S>,
}

pub struct ShardedMap<K, V, S> {
    hasher: S,
    shards: Vec<Mutex<Shard<K, V, S>>>,
}

impl<K, V, S> ShardedMap<K, V, S> where S: BuildHasher + Default {
    pub fn with_capacity(cap: usize) -> Self {
        let shards = 256;
        ShardedMap {
            hasher: default(),
            shards: (0..shards).map(|_| Mutex::new(Shard {
                arena: Arena::with_capacity(cap / shards),
                map: HashMap::with_capacity_and_hasher(cap / shards, default()),
            })).collect(),
        }
    }
    pub fn get<Q>(&self, key: Cow<Q>) -> &V where K: Borrow<Q> + Hash + Eq, Q: ?Sized + ToOwned<Owned=K> + Hash + Eq, V: Default {
        unsafe {
            let hash = self.hasher.hash_one(&key).rotate_right(16);
            let ref mut shard = *self.shards[(hash as usize) & (self.shards.len() - 1)].lock();
            match shard.map.raw_entry_mut().from_key(&*key) {
                RawEntryMut::Occupied(entry) => entry.into_mut().deref_unbounded(),
                RawEntryMut::Vacant(entry) => {
                    entry.insert(key.into_owned(), UncheckedRef::new(shard.arena.alloc(V::default()))).1.deref_unbounded()
                }
            }
        }
    }
}

unsafe impl<T: Sync> Send for UncheckedRef<T> {}

unsafe impl<T: Sync> Sync for UncheckedRef<T> {}