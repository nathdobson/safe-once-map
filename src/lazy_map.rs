use std::borrow::{Borrow, Cow};
use std::default::default;
use std::hash::Hash;
use std::marker::Tuple;
use safe_once::once::Once;
use safe_once::raw::RawOnce;
use crate::once_map::OnceMap;
use crate::simple_stable_map::SimpleStableMap;
use crate::stable_map::StableMap;

pub struct LazyFn<K, V, F, M> {
    callback: F,
    map: OnceMap<K, V, M>,
}

impl<K, V, F, M, RO> LazyFn<K, V, F, M> where
    M: Default + StableMap<Key=K, Value=Once<RO, V>>,
    RO: RawOnce,
    K: Tuple + Eq + Hash + Clone,
    F: Fn<K, Output=V> {
    pub fn new(callback: F) -> Self {
        LazyFn { callback, map: default() }
    }
    pub fn get(&self, key: K) -> &V {
        self.map.get(Cow::Borrowed(&key)).get_or_init(|| self.callback.call(key))
    }
}

impl<K, V, F, M, RO> FnOnce<K> for LazyFn<K, V, F, M> where K: Tuple + Eq + Hash + Clone, M: Default + StableMap<Key=K, Value=Once<RO, V>>, RO: RawOnce, V: Clone, F: Fn<K, Output=V> {
    type Output = V;
    extern "rust-call" fn call_once(self, args: K) -> Self::Output {
        self.call(args)
    }
}

impl<'m, K, V, F, M, RO> FnMut<K> for LazyFn<K, V, F, M> where K: Tuple + Eq + Hash + Clone, M: Default + StableMap<Key=K, Value=Once<RO, V>>, RO: RawOnce, V: Clone, F: Fn<K, Output=V> {
    extern "rust-call" fn call_mut(&mut self, args: K) -> Self::Output {
        self.call(args)
    }
}

impl<'m, K, V, F, M, RO> Fn<K> for LazyFn<K, V, F, M> where K: Tuple + Eq + Hash + Clone, M: Default + StableMap<Key=K, Value=Once<RO, V>>, RO: RawOnce, V: Clone, F: Fn<K, Output=V> {
    extern "rust-call" fn call(&self, args: K) -> Self::Output {
        self.get(args).clone()
    }
}
