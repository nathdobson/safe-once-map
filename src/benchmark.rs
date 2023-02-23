use std::any::type_name;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
use test::{Bencher, black_box};
use chashmap::CHashMap;
use concurrent_map::{ConcurrentMap, Minimum};
use dashmap::DashMap;
use itertools::Itertools;
use linreg::linear_regression_of;
use rand::Rng;
use safe_once::sync::OnceLock;
use crate::concurrent_fixed::ConcFixedHashMap;
use crate::load_factor::{BuildLoadFactor, BuildSampledLoadFactor, LoadFactor, NoLoadFactor, SampledLoadFactor};
use crate::sharded_map::ShardedMap;

trait BenchMap<K, V>: Send + Clone {
    fn with_capacity(x: usize) -> Self;
    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V;
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Clone + Sync + Send> BenchMap<K, V> for Arc<ConcFixedHashMap<K, V, RandomState, SampledLoadFactor>> {
    fn with_capacity(x: usize) -> Self {
        Arc::new(ConcFixedHashMap::with_capacity(x, BuildSampledLoadFactor))
    }
    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V {
        self.entry(Cow::Owned(key)).unwrap().1.get_or_init(value).to_owned().clone()
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Clone + Sync + Send> BenchMap<K, V> for Arc<ShardedMap<K, OnceLock<V>, RandomState>> {
    fn with_capacity(x: usize) -> Self {
        Arc::new(ShardedMap::with_capacity(x))
    }

    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V {
        self.get(Cow::Owned(key)).get_or_init(value).clone()
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Clone + Sync + Send> BenchMap<K, V> for Arc<DashMap<K, V>> {
    fn with_capacity(x: usize) -> Self { Arc::new(DashMap::with_capacity(x)) }
    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V {
        self.entry(key).or_insert_with(value).value().clone()
    }
}

impl<K: Eq + Hash + Clone + Sync + Send, V: Clone + Sync + Send> BenchMap<K, V> for Arc<CHashMap<K, V>> {
    fn with_capacity(x: usize) -> Self { Arc::new(CHashMap::with_capacity(x)) }
    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V {
        if let Some(value) = self.get(&key) {
            return value.clone();
        }
        let mut result: Cell<Option<V>> = Cell::new(None);
        self.upsert(key.clone(), || {
            let value = value();
            result.set(Some(value.clone()));
            value
        }, |value| {
            result.set(Some(value.clone()))
        });
        result.into_inner().unwrap()
    }
}

impl<K: Eq + Clone + Sync + Send + Minimum, V: Clone + Sync + Send + Eq> BenchMap<K, V> for ConcurrentMap<K, V> {
    fn with_capacity(x: usize) -> Self { ConcurrentMap::default() }
    fn get_or_init(&self, key: K, value: impl FnOnce() -> V) -> V {
        if let Some(value) = self.get(&key) {
            return value;
        }
        let value = value();
        let ignored_ = self.cas(key, None, Some(value.clone()));
        return value.clone();
    }
}

struct Benchmark {
    name: String,
    times: Vec<Duration>,
    callback: Box<dyn FnMut() -> Duration>,
}

impl Benchmark {
    fn new<M: BenchMap<i64, i64> + 'static>(name: String) -> Self {
        Benchmark {
            name,
            times: vec![],
            callback: box || {
                let threads: usize = 8;
                let ops: usize = 1 * 1024 * 1024;
                let keys: usize = 256 * 1024;
                let capacity = 2 * keys;
                let mut rng = rand::thread_rng();
                let key_seq: Vec<i64> = (0..ops).map(|_| rng.gen_range(0i64..keys as i64)).collect();
                let m = M::with_capacity(capacity);
                let b = Arc::new(Barrier::new(threads + 1));
                let joins = key_seq.chunks(key_seq.len() / threads)
                    .map(|x| x.iter().cloned().collect::<Vec<_>>())
                    .map(|key_seq_for_thread| thread::spawn({
                        let m = m.clone();
                        let b = b.clone();
                        move || {
                            b.wait();
                            for i in key_seq_for_thread {
                                #[inline(never)]
                                fn run<M: BenchMap<i64, i64>>(m: &M, i: i64) -> i64 {
                                    m.get_or_init(i, || i)
                                }
                                assert_eq!(i as i64, black_box(run(black_box(&m), black_box(i as i64))));
                            }
                            b.wait();
                        }
                    }))
                    .collect::<Vec<_>>();
                b.wait();
                let start = Instant::now();
                b.wait();
                let end = Instant::now();
                for join in joins {
                    join.join().unwrap();
                }
                end - start
            },
        }
    }
    fn run(&mut self) {
        self.times.push((self.callback)())
    }
    fn print(&self) {
        // let tuples: Vec<(f64, f64)> =
        //     self.times.iter()
        //         .sorted()
        //         .enumerate()
        //         .map(|(i, d)| (i as f64, d.as_secs_f64()))
        //         .collect();
        // let tuples = &tuples[1..tuples.len() * 3 / 4];
        // let (slope, intercept) = linear_regression_of::<f64, f64, f64>(&tuples).unwrap();
        // let error = statistical::mean(&tuples.iter().map(|(x, y)| (slope * (*x as f64) + intercept - y).abs()).collect::<Vec<_>>());
        // let midpoint = slope * ((tuples.len() / 2) as f64) + intercept;
        // let error = Duration::from_secs_f64(error);
        // let midpoint = Duration::from_secs_f64(midpoint);
        let mut times = self.times.clone();
        times.sort();
        // println!("{}", times.iter().map(|x| format!("{:?}", x)).join("\t"));
        let start = times.len() / 10;
        let value = times[start];
        let error = Duration::from_secs_f64((times[start + 2].as_secs_f64() - value.as_secs_f64()).abs());
        println!("{:?}\t±\t{:?}\t{}", value, error, self.name);
        // let mut used = self.times[self.times.len() / 4..self.times.len()].iter().map(|x| x.as_secs_f64()).collect::<Vec<_>>();
        // used.sort_floats();
        // let used = &used[used.len() / 4..used.len() * 3 / 4];
        // let stddev = Duration::from_secs_f64(statistical::standard_deviation(&used, None));
        // let mean = Duration::from_secs_f64(statistical::mean(&used));
        // let median = Duration::from_secs_f64(statistical::median(&used));
        // println!("{}", self.times.iter().sorted().map(|x|x.as_secs_f64()*1000.0).join("\t"));
        // println!("{:?}±{:?} ({:?})\t{}", mean, stddev, median, self.name);
    }
}

#[test]
fn run_bench() {
    let mut benchmarks = vec![
        Benchmark::new::<Arc<ConcFixedHashMap<i64, i64, RandomState, SampledLoadFactor>>>("ConcFixedHashMap<...,SampledLoadPolicy>".to_string()),
        Benchmark::new::<Arc<ShardedMap<i64, OnceLock<i64>, RandomState>>>("ShardedMap".to_string()),
        // Benchmark::new::<Arc<DashMap<i64, i64>>>("DashMap".to_string()),
        // Benchmark::new::<Arc<CHashMap<i64, i64>>>("CHashMap".to_string()),
        // Benchmark::new::<ConcurrentMap<i64, i64>>("ConcurrentMap".to_string()),
    ];
    for i in 0..20 {
        for b in &mut benchmarks {
            b.run();
        }
    }
    for b in benchmarks {
        b.print()
    }
}
