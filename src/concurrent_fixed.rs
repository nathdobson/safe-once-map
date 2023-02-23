use std::borrow::{Borrow, Cow};
use std::collections::hash_map::RandomState;
use std::default::default;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{BuildHasher, Hash, Hasher};
use std::ptr::hash;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use cache_padded::CachePadded;
use safe_once::once::OnceEntry;
use safe_once::sync::OnceLock;
use crate::load_factor::{SampledLoadFactor, LoadFactor, BuildLoadFactor, BuildSampledLoadFactor};

struct Key<K> {
    hashcode: u64,
    key: K,
}

struct Bucket<K, V> {
    key: OnceLock<Key<K>>,
    value: OnceLock<V>,
}

pub struct ConcFixedHashMap<K, V, S = RandomState, L = SampledLoadFactor> {
    state: S,
    cap: usize,
    mask: usize,
    table: Vec<Bucket<K, V>>,
    load_policy: L,
}


#[derive(Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub struct CapacityError<Q>(pub Q);

impl<Q> Debug for CapacityError<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapacityError").finish()
    }
}

impl<Q> Display for CapacityError<Q> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Data structure capacity exceeded")
    }
}

impl<K, V, S, L> ConcFixedHashMap<K, V, S, L> where S: BuildHasher, K: Hash + Eq, L: LoadFactor {
    pub fn with_capacity<BL>(mut cap: usize, build_load_factor: BL) -> Self where S: Default, BL: BuildLoadFactor<LoadFactor=L> {
        cap = cap.next_power_of_two();
        let mask = cap - 1;
        ConcFixedHashMap {
            state: default(),
            cap,
            mask,
            table: (0..cap).map(|_| Bucket {
                key: OnceLock::new(),
                value: default(),
            }).collect(),
            load_policy: build_load_factor.build_load_factor(cap),
        }
    }
    pub fn entry<'q, Q>(&self, key: Cow<'q, Q>) -> Result<(&K, &OnceLock<V>), CapacityError<Cow<'q, Q>>> where K: Borrow<Q>, Q: ?Sized + ToOwned<Owned=K> + Hash + Eq {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);
        let hashcode = hasher.finish();
        let mut index = self.mask & hashcode as usize;
        let mut step = 0;
        loop {
            let bucket = &self.table[index];
            match bucket.key.lock() {
                OnceEntry::Occupied(other_key) => {
                    if other_key.hashcode == hashcode {
                        if other_key.key.borrow() == key.borrow() {
                            return Ok((&other_key.key, &bucket.value));
                        }
                    }
                }
                OnceEntry::Vacant(lock) => {
                    if let Err(CapacityError(())) = self.load_policy.try_insert(index) {
                        return Err(CapacityError(key));
                    }
                    return Ok((&lock.init(Key { hashcode, key: key.into_owned() }).key, &bucket.value));
                }
            }
            step += 1;
            index += step * step;
            index = index & self.mask;
        }
    }
    pub fn get<'q, Q>(&self, key: &'q Q) -> Result<&OnceLock<V>, CapacityError<Cow<'q, Q>>> where K: Borrow<Q>, Q: ?Sized + ToOwned<Owned=K> + Hash + Eq {
        Ok(self.entry(Cow::Borrowed(key))?.1)
    }
    pub fn is_full(&self) -> bool {
        self.load_policy.is_full()
    }
}

#[test]
fn test() {
    let x = ConcFixedHashMap::<String, String>::with_capacity(3, BuildSampledLoadFactor);
    assert_eq!(x.get("a").unwrap().get_or_init(|| "b".to_string()), "b");
    assert_eq!(x.get("a").unwrap().get_or_init(|| "c".to_string()), "b");
}

