use std::borrow::Cow;
use std::collections::hash_map::{Entry, RandomState};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rand::distributions::Alphanumeric;
use safe_once::sync::OnceLock;
use crate::concurrent::ConcHashMap;
use crate::load_factor::BuildSampledLoadFactor;
use crate::probing::BuildTriangularProbing;


type TestMap<K, V> = ConcHashMap<K, V, RandomState, BuildTriangularProbing>;

#[test]
fn test() {
    let x = TestMap::<String, OnceLock<i64>>::with_capacity(16, 2, 16);
    assert_eq!(&1, x.entry(Cow::Borrowed("a")).unwrap().1.get_or_init(|| 1));
    assert_eq!(&2, x.entry(Cow::Borrowed("b")).unwrap().1.get_or_init(|| 2));
    assert_eq!(&1, x.entry(Cow::Borrowed("a")).unwrap().1.get_or_init(|| 3));
    assert_eq!(&2, x.entry(Cow::Borrowed("b")).unwrap().1.get_or_init(|| 4));
}


#[test]
fn test_random() {
    #[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
    struct UniqueId(usize);
    impl Default for UniqueId {
        fn default() -> Self {
            static counter: AtomicUsize = AtomicUsize::new(0);
            UniqueId(counter.fetch_add(1, Relaxed))
        }
    }
    for count in 0..100 {
        for seed in 0..1000 {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut map = TestMap::<String, UniqueId>::with_capacity(4, 2, 256);
            let keyspace = rng.gen_range(1..100);
            let mut expected = HashMap::<String, UniqueId>::new();
            for _ in 0..count {
                let key = format!("{}", rng.gen_range(0..keyspace));
                let value = *map.entry(Cow::Borrowed(&key)).unwrap().1;
                match expected.entry(key) {
                    Entry::Occupied(old) => assert_eq!(value, *old.get()),
                    Entry::Vacant(v) => {
                        v.insert(value);
                    }
                }
            }
            println!("{:?} {:?}", expected.len(), map.last_slot_count() as f64);
        }
    }
}