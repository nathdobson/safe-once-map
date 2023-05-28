use std::default::default;
use std::ops::Index;
use safe_once::once::Once;
use safe_once::raw::RawOnce;

pub struct IndexArena<R: RawOnce, T> {
    arena: Vec<Once<R, Vec<T>>>,
}

fn arena_index(index: usize) -> (usize, usize) {
    let major = (index + 1).ilog2() as usize;
    let minor = index + 1 - (1 << major);
    (major, minor)
}

impl<R: RawOnce, T: Default> IndexArena<R, T> {
    pub fn new() -> Self {
        IndexArena {
            arena: (0..usize::BITS).map(|_| default()).collect(),
        }
    }
    pub fn get(&self, index: usize) -> &T {
        let (major, minor) = arena_index(index);
        &self.arena[major].get_or_init(|| (0..1 << major).map(|_| default()).collect())[minor]
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
