use std::borrow::{Borrow, Cow};
use std::collections::hash_map::RawEntryMut;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

pub unsafe trait RawMap<Q: ?Sized + ToOwned> {
    type Value;
    fn get_or_insert(
        &mut self,
        key: Cow<Q>,
        insert: impl FnOnce() -> Self::Value,
    ) -> &mut Self::Value;
}

unsafe impl<K, V, Q> RawMap<Q> for HashMap<K, V>
    where K: Hash + Eq,
          Q: ?Sized + ToOwned<Owned=K> + Hash + Eq,
          K: Borrow<Q> {
    type Value = V;
    fn get_or_insert(
        &mut self,
        key: Cow<Q>,
        insert: impl FnOnce() -> Self::Value,
    ) -> &mut Self::Value

    {
        match self.raw_entry_mut().from_key(&key) {
            RawEntryMut::Occupied(entry) => entry.into_mut(),
            RawEntryMut::Vacant(entry) => {
                entry.insert(key.into_owned(), insert()).1
            }
        }
    }
}

unsafe impl<K, V, Q> RawMap<Q> for BTreeMap<K, V>
    where K: Ord,
          Q: ?Sized + ToOwned<Owned=K> + Eq,
          K: Borrow<Q> {
    type Value = V;
    fn get_or_insert(
        &mut self,
        key: Cow<Q>,
        insert: impl FnOnce() -> Self::Value,
    ) -> &mut Self::Value

    {
        self.entry(key.into_owned()).or_insert_with(insert)
    }
}
