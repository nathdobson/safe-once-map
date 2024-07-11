use safe_once::api::once::Once;
use safe_once::api::raw::RawFused;
use safe_once::cell::{OnceCell, RawFusedCell};
use std::ops::Index;

pub struct IndexArena<R: RawFused, T> {
    arena: Vec<Once<R, Vec<T>>>,
}

fn arena_index(index: usize) -> (usize, usize) {
    let major = (index + 1).ilog2() as usize;
    let minor = index + 1 - (1 << major);
    (major, minor)
}

impl<R: RawFused, T: Default> IndexArena<R, T> {
    pub fn new() -> Self {
        IndexArena {
            arena: (0..usize::BITS).map(|_| Default::default()).collect(),
        }
    }
    pub fn get_or_init(&self, index: usize) -> &T {
        let (major, minor) = arena_index(index);
        &self.arena[major].get_or_init(|| (0..1 << major).map(|_| Default::default()).collect())
            [minor]
    }
    pub fn try_get(&self, index: usize) -> Option<&T> {
        let (major, minor) = arena_index(index);
        Some(&self.arena[major].get()?[minor])
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
fn test_arena() {
    let arena = IndexArena::<RawFusedCell, OnceCell<String>>::new();
    arena.get_or_init(123).set("Asd".to_string()).unwrap();
}
