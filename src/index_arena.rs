use std::default::default;
use std::ops::Index;
use safe_once::once::Once;
use safe_once::raw::RawOnce;

pub struct IndexArena<R: RawOnce, T> {
    arena: Vec<Once<R, Vec<T>>>,
    shards: usize,
}

fn arena_index(index: usize) -> (usize, usize) {
    let major = (index + 1).ilog2() as usize;
    let minor = index + 1 - (1 << major);
    (major, minor)
}

fn sharded_index(shard: usize, shards: usize, index: usize) -> (usize, usize) {
    let (major, minor) = arena_index(index);
    (major, (1 << major) * shard + minor)
}

impl<R: RawOnce, T: Default> IndexArena<R, T> {
    pub fn new(shards: usize) -> Self {
        IndexArena {
            arena: (0..usize::BITS).map(|_| default()).collect(),
            shards,
        }
    }
    pub fn get(&self, shard: usize, index: usize) -> &T {
        let (major, minor) = sharded_index(shard, self.shards, index);
        &self.arena[major].get_or_init(|| (0..self.shards << major).map(|_| default()).collect())[minor]
    }
}

#[test]
fn test_arena_index() {
    assert_eq!((0, 0), arena_index(0));
    assert_eq!((1, 0), arena_index(1));
    assert_eq!((1, 1), arena_index(2));
    assert_eq!((2, 0), arena_index(3));
    assert_eq!((2, 1), arena_index(4));
    assert_eq!((2, 2), arena_index(5));
    assert_eq!((2, 3), arena_index(6));
    assert_eq!((3, 0), arena_index(7));
}

#[test]
fn test_shard_index() {
    assert_eq!((0, 0), sharded_index(0, 3, 0));
    assert_eq!((1, 0), sharded_index(0, 3, 1));
    assert_eq!((1, 1), sharded_index(0, 3, 2));
    assert_eq!((2, 0), sharded_index(0, 3, 3));
    assert_eq!((2, 1), sharded_index(0, 3, 4));
    assert_eq!((2, 2), sharded_index(0, 3, 5));
    assert_eq!((2, 3), sharded_index(0, 3, 6));
    assert_eq!((3, 0), sharded_index(0, 3, 7));

    assert_eq!((0, 1), sharded_index(1, 3, 0));
    assert_eq!((1, 2), sharded_index(1, 3, 1));
    assert_eq!((1, 3), sharded_index(1, 3, 2));
    assert_eq!((2, 4), sharded_index(1, 3, 3));
    assert_eq!((2, 5), sharded_index(1, 3, 4));
    assert_eq!((2, 6), sharded_index(1, 3, 5));
    assert_eq!((2, 7), sharded_index(1, 3, 6));
    assert_eq!((3, 8), sharded_index(1, 3, 7));

    assert_eq!((0, 2), sharded_index(2, 3, 0));
    assert_eq!((1, 4), sharded_index(2, 3, 1));
    assert_eq!((1, 5), sharded_index(2, 3, 2));
    assert_eq!((2, 8), sharded_index(2, 3, 3));
    assert_eq!((2, 9), sharded_index(2, 3, 4));
    assert_eq!((2, 10), sharded_index(2, 3, 5));
    assert_eq!((2, 11), sharded_index(2, 3, 6));
    assert_eq!((3, 16), sharded_index(2, 3, 7));
}