use std::borrow::Borrow;
use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::hash::Hash;
use colosseum::sync::Arena;
use parking_lot::{Mutex, RawMutex};
use safe_once::sync::OnceLock;
use crate::stable_map::{StableMap, UnboundedRef};


pub type OnceLockMap<K, V, RM = RawMutex, M = HashMap<K, UnboundedRef<OnceLock<V>>>> = StableMap<K, OnceLock<V>, RM, M>;

#[test]
fn test() {
    let map = OnceLockMap::<String, String>::new();
    assert_eq!("b", *map["a"].get_or_init(|| "b".to_string()));
    assert_eq!("b", *map["a"].get_or_init(|| "c".to_string()));
}