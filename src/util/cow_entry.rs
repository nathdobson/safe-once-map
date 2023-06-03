use std::borrow::{Borrow, Cow};
use std::collections::hash_map::{RawEntryMut, RawOccupiedEntryMut, RawVacantEntryMut};
use std::collections::HashMap;
use std::default::default;
use std::hash::{BuildHasher, Hash};

pub trait CowEntry {
    type Key: Hash + Eq;
    type Value;
    type State: BuildHasher;
    fn cow_entry_mut<'m, 'q, Q>(
        &'m mut self,
        key: Cow<'q, Q>,
    ) -> CowEntryMut<'m, 'q, Self::Key, Self::Value, Self::State, Q>
        where
            Self::Key: Borrow<Q>,
            Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>;
}

impl<K, V, S> CowEntry for HashMap<K, V, S> where K: Hash + Eq, S: BuildHasher {
    type Key = K;
    type Value = V;
    type State = S;

    fn cow_entry_mut<'m, 'q, Q>(
        &'m mut self, key: Cow<'q, Q>,
    ) -> CowEntryMut<'m, 'q, Self::Key, Self::Value, Self::State, Q>
        where
            Self::Key: Borrow<Q>,
            Q: ?Sized + Hash + Eq + ToOwned<Owned=Self::Key>
    {
        CowEntryMut::new(self, key)
    }
}

pub enum CowEntryMut<'m, 'q, K, V, S, Q: ?Sized + ToOwned<Owned=K>> {
    Occupied(CowOccupiedEntryMut<'m, 'q, K, V, S, Q>),
    Vacant(CowVacantEntryMut<'m, 'q, K, V, S, Q>),
}

pub struct CowOccupiedEntryMut<'m, 'q, K, V, S, Q: ?Sized + ToOwned<Owned=K>> {
    entry: RawOccupiedEntryMut<'m, K, V, S>,
    cow: Cow<'q, Q>,
}

pub struct CowVacantEntryMut<'m, 'q, K, V, S, Q: ?Sized + ToOwned<Owned=K>> {
    entry: RawVacantEntryMut<'m, K, V, S>,
    cow: Cow<'q, Q>,
}

impl<'m, 'q, K, V, S, Q> CowEntryMut<'m, 'q, K, V, S, Q> where
    K: Hash + Eq + Borrow<Q>,
    Q: ?Sized + Hash + Eq + ToOwned<Owned=K>,
    S: BuildHasher {
    pub fn new(map: &'m mut HashMap<K, V, S>, query: Cow<'q, Q>) -> Self {
        match map.raw_entry_mut().from_key(&query) {
            RawEntryMut::Occupied(o) => CowEntryMut::Occupied(CowOccupiedEntryMut {
                entry: o,
                cow: query,
            }),
            RawEntryMut::Vacant(v) => CowEntryMut::Vacant(CowVacantEntryMut {
                entry: v,
                cow: query,
            }),
        }
    }
    pub fn or_insert(self, default: V) -> &'m mut V {
        match self {
            CowEntryMut::Occupied(o) => o.into_mut(),
            CowEntryMut::Vacant(v) => v.insert(default),
        }
    }
    pub fn or_insert_with(self, default: impl FnOnce() -> V) -> &'m mut V {
        match self {
            CowEntryMut::Occupied(o) => o.into_mut(),
            CowEntryMut::Vacant(v) => v.insert(default()),
        }
    }
    pub fn or_insert_with_key(self, default: impl FnOnce(&Q) -> V) -> &'m mut V {
        match self {
            CowEntryMut::Occupied(o) => o.into_mut(),
            CowEntryMut::Vacant(v) => {
                let default = default(&*v.cow);
                v.insert(default)
            }
        }
    }
    pub fn query(&self) -> &Q {
        match self {
            CowEntryMut::Occupied(o) => o.query(),
            CowEntryMut::Vacant(v) => v.query(),
        }
    }
    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        match self {
            CowEntryMut::Occupied(mut o) => {
                f(o.get_mut());
                CowEntryMut::Occupied(o)
            }
            CowEntryMut::Vacant(v) => {
                CowEntryMut::Vacant(v)
            }
        }
    }
    pub fn or_default(self) -> &'m mut V where V: Default {
        self.or_insert_with(default)
    }
}

impl<'m, 'q, K, V, S, Q> CowOccupiedEntryMut<'m, 'q, K, V, S, Q> where
    K: Hash + Eq + Borrow<Q>,
    Q: ?Sized + Hash + Eq + ToOwned<Owned=K>,
    S: BuildHasher {
    pub fn into_mut(self) -> &'m mut V {
        self.entry.into_mut()
    }
    pub fn query(&self) -> &Q { &*self.cow }
    pub fn key(&self) -> &K { self.entry.key() }
    pub fn get_mut(&mut self) -> &mut V {
        self.entry.get_mut()
    }
    pub fn insert(&mut self, value: V) -> V {
        self.entry.insert(value)
    }
}

impl<'m, 'q, K, V, S, Q> CowVacantEntryMut<'m, 'q, K, V, S, Q> where
    K: Hash + Eq + Borrow<Q>,
    Q: ?Sized + Hash + Eq + ToOwned<Owned=K>,
    S: BuildHasher {
    pub fn insert(self, value: V) -> &'m mut V {
        self.entry.insert(self.cow.into_owned(), value).1
    }
    pub fn query(&self) -> &Q { &*self.cow }
}
