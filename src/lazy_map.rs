use std::borrow::{Borrow, Cow};
use std::default::default;
use std::hash::{BuildHasher, Hash};
use std::marker::{PhantomData, Tuple};
use lock_api::RawMutex;
use safe_once::cell::OnceCell;
use safe_once::once::Once;
use safe_once::raw::RawOnce;
use crate::stable_map::StableMap;

pub struct LazyMap<K, V, F, S, RO: RawOnce, RM> {
    callback: F,
    map: StableMap<K, OnceCell<V>, S, RO, RM>,
    phantom: PhantomData<(K, V)>,
}

//
impl<K, V, F, S, RO: RawOnce, RM> LazyMap<K, V, F, S, RO, RM> where
    RO: RawOnce,
    K: Tuple + Eq + Hash + Clone,
    F: Fn<K, Output=V>,
    S: Default + BuildHasher,
    RM: RawMutex,
{
    pub fn new(callback: F) -> Self {
        LazyMap { callback, map: default(), phantom: PhantomData }
    }
    pub fn get(&self, key: K) -> &V {
        self.map[&key].get_or_init(|| self.callback.call(key))
    }
}

impl<K, V, F, S, RO: RawOnce, RM> FnOnce<K> for LazyMap<K, V, F, S, RO, RM> where
    K: Tuple + Eq + Hash + Clone,
    RO: RawOnce,
    V: Clone,
    F: Fn<K, Output=V>,
    S: Default + BuildHasher,
    RM: RawMutex
{
    type Output = V;
    extern "rust-call" fn call_once(self, args: K) -> Self::Output {
        self.get(args).clone()
    }
}

impl<K, V, F, S, RO: RawOnce, RM> FnMut<K> for LazyMap<K, V, F, S, RO, RM> where
    K: Tuple + Eq + Hash + Clone,
    RO: RawOnce,
    V: Clone,
    F: Fn<K, Output=V>,
    S: Default + BuildHasher,
    RM: RawMutex
{
    extern "rust-call" fn call_mut(&mut self, args: K) -> Self::Output {
        self.get(args).clone()
    }
}

impl<K, V, F, S, RO: RawOnce, RM> Fn<K> for LazyMap<K, V, F, S, RO, RM> where
    K: Tuple + Eq + Hash + Clone,
    RO: RawOnce,
    V: Clone,
    F: Fn<K, Output=V>,
    S: Default + BuildHasher,
    RM: RawMutex
{
    extern "rust-call" fn call(&self, args: K) -> Self::Output {
        self.get(args).clone()
    }
}
