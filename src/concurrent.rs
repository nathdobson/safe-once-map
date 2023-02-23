use std::borrow::{Borrow, Cow};
use std::collections::hash_map::RandomState;
use std::default::default;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasher, Hash};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use safe_once::lazy::Lazy;
use safe_once::once::{OnceEntry, OnceGuard};
use safe_once::sync::{LazyLock, OnceLock, RawOnceLock};
use crate::concurrent_fixed::{CapacityError, ConcFixedHashMap};
use crate::load_factor::{BuildLoadFactor, BuildSampledLoadFactor, LoadFactor, SampledLoadFactor};
use crate::probing::{BuildProbing, BuildTriangularProbing, Probing, TriangularProbing};

#[derive(Clone)]
struct Key<K> {
    hash: u64,
    key: K,
}

struct Bucket<K, V> {
    key: OnceLock<Key<K>>,
    value: V,
}

#[derive(Copy, Clone)]
struct Redirect {
    depth: usize,
    index: usize,
}

enum Slot<K, V> {
    Redirect(Redirect),
    Bucket(Bucket<K, V>),
}

struct Layer<K, V, P> {
    table: Vec<Slot<K, V>>,
    probing: P,
}

pub struct ConcHashMap<K, V, S, P: BuildProbing> {
    caps: Vec<usize>,
    layers: Vec<OnceLock<Layer<K, V, P::Probing>>>,
    depth: AtomicUsize,
    build_hasher: S,
    build_probing: P,
    load_factor: AtomicUsize,
}

impl<K, V, S, P> ConcHashMap<K, V, S, P> where K: Eq + Hash, S: BuildHasher + Default, P: BuildProbing + Default, V: Default {
    pub fn with_capacity(mut cap_init: usize, cap_ratio: usize, mut cap_max: usize) -> Self {
        assert!(cap_init.is_power_of_two());
        assert!(cap_ratio.is_power_of_two());
        assert!(cap_max.is_power_of_two());
        let mut caps = vec![];
        let mut cap = cap_init;
        while cap <= cap_max {
            caps.push(cap);
            cap *= cap_ratio;
        }
        ConcHashMap {
            layers: caps.iter().map(|_| {
                OnceLock::new()
            }).collect(),
            caps,
            depth: AtomicUsize::new(0),
            build_hasher: default(),
            build_probing: default(),
            load_factor: todo!(),
        }
    }
    fn bucket<'a>(&'a self, slot: &'a Slot<K, V>) -> &'a Bucket<K, V> {
        match slot {
            Slot::Redirect(Redirect { depth, index }) => match &self.layer(*depth).unwrap().table[*index] {
                Slot::Redirect { .. } => panic!("Double redirect"),
                Slot::Bucket(bucket) => bucket
            },
            Slot::Bucket(bucket) => bucket
        }
    }
    fn layer(&self, depth: usize) -> Result<&Layer<K, V, P::Probing>, CapacityError<()>> {
        Ok(self.layers.get(depth).ok_or(CapacityError(()))?.get_or_init(|| {
            let cap = self.caps[depth];
            let table = (0..self.caps[depth]).map(|_| Slot::Bucket(Bucket {
                key: OnceLock::new(),
                value: V::default(),
            })).collect();
            let mut layer = Layer {
                table,
                probing: self.build_probing.build_probing(cap),
            };
            if let Some(prev_depth) = depth.checked_sub(1) {
                let prev_layer = self.layer(prev_depth).unwrap();
                'copy: for (index, slot) in prev_layer.table.iter().enumerate() {
                    let bucket = self.bucket(slot);
                    let redirect = match slot {
                        Slot::Redirect(redirect) => *redirect,
                        Slot::Bucket(_) => Redirect { depth: prev_depth, index },
                    };
                    if let Some(key) = bucket.key.get() {
                        for probe in layer.probing.probe(key.hash) {
                            match &layer.table[probe] {
                                Slot::Bucket(_) => {
                                    layer.table[probe] = Slot::Redirect(redirect);
                                    continue 'copy;
                                }
                                _ => {}
                            }
                        }
                        panic!("Failed to probe when copying.");
                    }
                }
            }
            self.depth.store(depth, Relaxed);
            layer
        }))
    }
    pub fn entry<'q, Q>(&self, mut key: Cow<'q, Q>) -> Result<(&K, &V), CapacityError<Cow<'q, Q>>> where K: Borrow<Q>, Q: ?Sized + ToOwned<Owned=K> + Hash + Eq {
        let hash = self.build_hasher.hash_one(&key);
        let mut depth = self.depth.load(Relaxed);
        'main: loop {
            let layer: &Layer<K, V, P::Probing> = match self.layer(depth) {
                Err(CapacityError(())) => {
                    return Err(CapacityError(key));
                }
                Ok(layer) => layer,
            };
            for probe in layer.probing.probe(hash) {
                let slot: &Slot<K, V> = &layer.table[probe];
                let bucket = self.bucket(slot);
                let lock: OnceEntry<RawOnceLock, Key<K>> = bucket.key.lock();
                let key_ref;
                match lock {
                    OnceEntry::Occupied(occupied) => {
                        if !(occupied.hash == hash && occupied.key.borrow() == key.borrow()) {
                            continue;
                        }
                        key_ref = occupied;
                    }
                    OnceEntry::Vacant(vacant) => {
                        todo!();
                        // if let Err(CapacityError(())) = layer.load_factor.try_insert(probe) {
                        //     break;
                        // }
                        key_ref = vacant.init(Key { key: key.into_owned(), hash });
                    }
                }
                return Ok((&key_ref.key, &bucket.value));
            }
            depth = self.depth.load(Relaxed).max(depth + 1);
            continue 'main;
        }
    }
    // pub fn depth(&self) -> usize {
    //     let depth = self.depth.load(Relaxed);
    //     for depth in depth..self.layers.len() {
    //         if self.layers[depth].try_get().is_none() {
    //             return depth.saturating_sub(1);
    //         }
    //     }
    //     self.layers.len()
    // }
    pub fn total_slot_count(&self) -> usize {
        self.layers.iter().flat_map(|x| x.try_get().map(|x| x.table.len())).sum()
    }
    pub fn last_slot_count(&self) -> usize {
        self.layers.iter().flat_map(|x| x.try_get().map(|x| x.table.len())).max().unwrap_or(0)
    }
    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item=(&K, &V)> {
        self.layers.iter().flat_map(|layer| {
            layer.try_get().into_iter().flat_map(|layer|
                layer.table.iter().flat_map(|x| match x {
                    Slot::Redirect(_) => None,
                    Slot::Bucket(bucket) => bucket.key.try_get().map(|key| {
                        (&key.key, &bucket.value)
                    })
                })
            )
        })
    }
}

impl<K, V, S, P> Debug for ConcHashMap<K, V, S, P> where K: Eq + Hash + Debug, S: BuildHasher + Default, P: BuildProbing + Default, V: Default + Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
