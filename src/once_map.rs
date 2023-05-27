use std::borrow::{Borrow, Cow};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;
use safe_once::once::Once;
use safe_once::raw::RawOnce;
use crate::stable_map::StableMap;

pub struct OnceMap<K, V, M> {
    map: M,
    phantom: PhantomData<(K, V)>,
}

impl<K, V, RO, M> Default for OnceMap<K, V, M> where M: Default + StableMap<Key=K, Value=Once<RO, V>>, RO: RawOnce {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, RO, M> OnceMap<K, V, M> where M: StableMap<Key=K, Value=Once<RO, V>>, RO: RawOnce {
    pub fn new() -> Self where M: Default {
        OnceMap {
            map: M::default(),
            phantom: PhantomData,
        }
    }
    pub fn get<'m, Q>(&'m self, key: Cow<Q>) -> &'m Once<RO, V>
        where Q: ?Sized + Hash + Eq + ToOwned<Owned=K>,
              K: Hash + Eq + Borrow<Q>,
              RO: 'm {
        self.map.get_or_insert(key)
    }
}

impl<'q, K, V, RO, M, Q> Index<&'q Q> for OnceMap<K, V, M>
    where Q: ?Sized + Hash + Eq + ToOwned<Owned=K>,
          K: Hash + Eq + Borrow<Q>,
          M: StableMap<Key=K, Value=Once<RO, V>>,
          RO: RawOnce {
    type Output = Once<RO, V>;
    fn index(&self, index: &'q Q) -> &Self::Output {
        self.get(Cow::Borrowed(index))
    }
}