use std::borrow::Cow;
use std::collections::HashMap;
use std::default::default;
use std::marker::PhantomData;
use std::ops::Index;
use colosseum::unsync::Arena;
use lock_api::RawMutex;
use parking_lot::Mutex;
use safe_once::unsync::OnceCell;
use crate::raw_map::RawMap;

pub struct UnboundedRef<T: ?Sized>(*const T);

unsafe impl<T: ?Sized + Sync> Send for UnboundedRef<T> {}

unsafe impl<T: ?Sized + Sync> Sync for UnboundedRef<T> {}

impl<T: ?Sized> UnboundedRef<T> {
    pub fn new(x: &T) -> Self { UnboundedRef(x) }
    pub unsafe fn deref_unbounded(&self) -> &T { &*self.0 }
    pub unsafe fn deref_escape<'a, 'b>(&'a self) -> &'b T { &*self.0 }
}

impl<T: ?Sized> Clone for UnboundedRef<T> {
    fn clone(&self) -> Self { UnboundedRef(self.0) }
}

pub struct StableMap<K, V, RM, M = HashMap<K, UnboundedRef<V>>> {
    arena: Arena<V>,
    map: lock_api::Mutex<RM, M>,
    phantom: PhantomData<K>,
}

impl<K, V, RM: RawMutex, M> StableMap<K, V, RM, M> {
    pub fn new() -> Self where M: Default {
        StableMap {
            arena: Arena::new(),
            map: default(),
            phantom: PhantomData,
        }
    }
    pub fn get<Q: ?Sized>(
        &self,
        q: Cow<Q>,
        insert: impl FnOnce() -> V,
    ) -> &V where Q: ToOwned<Owned=K>, M: RawMap<Q, Value=UnboundedRef<V>> {
        unsafe {
            self.map.lock().get_or_insert(q, || {
                UnboundedRef::new(self.arena.alloc(insert()))
            }).deref_escape()
        }
    }
}

impl<'q, Q, K, V, RM: RawMutex, M> Index<&'q Q> for StableMap<K, V, RM, M>
    where Q: ?Sized + ToOwned<Owned=K>, M: RawMap<Q, Value=UnboundedRef<V>>, V: Default {
    type Output = V;
    fn index(&self, index: &'q Q) -> &Self::Output {
        self.get(Cow::Borrowed(index), default)
    }
}